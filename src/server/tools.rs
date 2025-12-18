//! MCP Server tools for astrological calculations

use std::sync::Arc;

use chrono::NaiveDate;
use rmcp::{
    RoleServer, ServerHandler,
    model::{
        CallToolRequestParam, CallToolResult, Content, Implementation, ListToolsResult,
        PaginatedRequestParam, ProtocolVersion, ServerCapabilities, ServerInfo, Tool,
    },
    schemars::{self, schema_for},
    service::RequestContext,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::ephemeris::{
    calc_all_planets, calc_houses, calc_planet_position, calc_sun_moon_angle, date_to_julian_day,
    find_next_full_moon, find_next_new_moon, find_next_sign_ingress, find_next_station,
    house_system_name, julian_day_to_date, local_datetime_to_julian_day, planet_in_house,
    HOUSE_PLACIDUS,
};
use crate::models::{
    find_aspect, Aspect, DateRange, GetDailyTransitsResponse, GetLunarInfoResponse,
    GetRetrogradeStatusResponse, GetTransitReportResponse, HouseCusps, LunarCycle, LunarEvent,
    LunarPhase, LunarPhaseName, MajorEvent, NatalChart, NatalChartSummary, Planet, PlanetPosition,
    RetrogradeInfo, StoreNatalChartRequest, StoreNatalChartResponse, Transit, UpcomingRetrograde,
    VoidOfCourse, ZodiacPosition,
};
use crate::storage::Storage;

/// Input for storing a natal chart
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct StoreNatalChartInput {
    #[schemars(description = "Name of the person")]
    pub name: String,
    #[schemars(description = "Birth date in YYYY-MM-DD format")]
    pub birth_date: String,
    #[schemars(description = "Birth time in HH:MM:SS format")]
    pub birth_time: String,
    #[schemars(description = "Birth location name")]
    pub birth_location: String,
    #[schemars(description = "Latitude of birth location")]
    pub latitude: f64,
    #[schemars(description = "Longitude of birth location")]
    pub longitude: f64,
    #[schemars(description = "Timezone identifier (e.g., 'America/Panama')")]
    pub timezone: String,
}

/// Input for daily transits
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct DailyTransitsInput {
    #[schemars(description = "Date to get transits for in YYYY-MM-DD format")]
    pub date: String,
}

/// Input for retrograde status
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct RetrogradeStatusInput {
    #[schemars(description = "Date to check retrograde status for in YYYY-MM-DD format")]
    pub date: String,
    #[schemars(description = "Whether to include upcoming retrograde periods (default: true)")]
    pub include_upcoming: Option<bool>,
    #[schemars(description = "Number of days to look ahead for upcoming retrogrades (default: 90)")]
    pub days_ahead: Option<i64>,
}

/// Input for lunar info
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct LunarInfoInput {
    #[schemars(description = "Date to get lunar information for in YYYY-MM-DD format")]
    pub date: String,
}

/// Input for transit report
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct TransitReportInput {
    #[schemars(description = "Start date in YYYY-MM-DD format")]
    pub start_date: String,
    #[schemars(description = "End date in YYYY-MM-DD format")]
    pub end_date: String,
    #[schemars(description = "Whether to include minor aspects (default: false)")]
    pub include_minor_aspects: Option<bool>,
}

/// Input for getting a natal chart
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct GetNatalChartInput {
    #[schemars(description = "Name of the natal chart to retrieve")]
    pub name: String,
}

/// Input for deleting a natal chart
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct DeleteNatalChartInput {
    #[schemars(description = "Name of the natal chart to delete")]
    pub name: String,
    #[schemars(description = "Birth date of the chart to delete (for confirmation) in YYYY-MM-DD format")]
    pub birth_date: String,
}

/// Input for searching natal charts
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct SearchNatalChartsInput {
    #[schemars(description = "Search query - matches against name (case-insensitive, partial match)")]
    pub query: String,
}

/// Input for compatibility/synastry analysis
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct GetCompatibilityInput {
    #[schemars(description = "Name of the first person's natal chart")]
    pub person1_name: String,
    #[schemars(description = "Name of the second person's natal chart")]
    pub person2_name: String,
    #[schemars(description = "Include minor aspects (sextile, quincunx) - default: false")]
    pub include_minor_aspects: Option<bool>,
}

fn schema_to_value<T: schemars::JsonSchema>() -> Arc<serde_json::Map<String, Value>> {
    let schema = schema_for!(T);
    let value = serde_json::to_value(schema).unwrap();
    if let Value::Object(map) = value {
        Arc::new(map)
    } else {
        Arc::new(serde_json::Map::new())
    }
}

fn empty_schema() -> Arc<serde_json::Map<String, Value>> {
    let mut map = serde_json::Map::new();
    map.insert("type".into(), Value::String("object".into()));
    map.insert("properties".into(), Value::Object(serde_json::Map::new()));
    Arc::new(map)
}

/// MCP Server for astrological calculations
#[derive(Clone)]
pub struct StelliumServer {
    storage: Arc<Storage>,
}

impl StelliumServer {
    pub fn new() -> Self {
        let storage = Arc::new(Storage::new().expect("Failed to initialize storage"));
        Self { storage }
    }

    fn store_natal_chart(&self, input: StoreNatalChartInput) -> String {
        let request = StoreNatalChartRequest {
            name: input.name,
            birth_date: input.birth_date,
            birth_time: input.birth_time,
            birth_location: input.birth_location,
            latitude: input.latitude,
            longitude: input.longitude,
            timezone: input.timezone,
        };

        let date = match request.parse_date() {
            Ok(d) => d,
            Err(e) => return json!({
                "success": false,
                "error": format!("Invalid date format: {}. Expected YYYY-MM-DD", e)
            }).to_string(),
        };

        let time = match request.parse_time() {
            Ok(t) => t,
            Err(e) => return json!({
                "success": false,
                "error": format!("Invalid time format: {}. Expected HH:MM:SS", e)
            }).to_string(),
        };

        let julian_day = match local_datetime_to_julian_day(date, time, &request.timezone) {
            Ok(jd) => jd,
            Err(e) => return json!({
                "success": false,
                "error": format!("Timezone error: {}", e)
            }).to_string(),
        };

        let mut chart = NatalChart::new(&request);

        // Calculate house positions (Placidus by default)
        let house_data = match calc_houses(julian_day, request.latitude, request.longitude, HOUSE_PLACIDUS) {
            Ok(h) => h,
            Err(e) => return json!({
                "success": false,
                "error": format!("Failed to calculate houses: {}", e)
            }).to_string(),
        };

        // Store house cusps
        chart.houses = Some(HouseCusps {
            cusps: house_data.cusps.iter().map(|&lon| ZodiacPosition::from_longitude(lon)).collect(),
            system: house_system_name(HOUSE_PLACIDUS).to_string(),
        });

        chart.ascendant = Some(ZodiacPosition::from_longitude(house_data.ascendant));
        chart.midheaven = Some(ZodiacPosition::from_longitude(house_data.midheaven));
        chart.vertex = Some(ZodiacPosition::from_longitude(house_data.vertex));

        // Calculate planetary positions
        let positions = match calc_all_planets(julian_day) {
            Ok(p) => p,
            Err(e) => return json!({
                "success": false,
                "error": format!("Failed to calculate planetary positions: {}", e)
            }).to_string(),
        };

        for (planet, position) in positions {
            let zodiac_pos = position.to_zodiac_position();
            let house = planet_in_house(position.longitude, &house_data.cusps);

            // Store in legacy format for compatibility
            chart.planets.insert(planet, zodiac_pos.clone());

            // Store in new format with house placement
            chart.planet_positions.insert(planet, PlanetPosition {
                position: zodiac_pos,
                house,
                is_retrograde: position.is_retrograde,
            });
        }

        if let Err(e) = self.storage.save_chart(chart.clone()) {
            return json!({
                "success": false,
                "error": format!("Failed to save chart: {}", e)
            }).to_string();
        }

        let response = StoreNatalChartResponse {
            success: true,
            message: "Natal chart stored successfully".to_string(),
            natal_chart: NatalChartSummary::from(&chart),
        };

        serde_json::to_string_pretty(&response).unwrap()
    }

    fn get_daily_transits(&self, input: DailyTransitsInput) -> String {
        let parsed_date = match NaiveDate::parse_from_str(&input.date, "%Y-%m-%d") {
            Ok(d) => d,
            Err(e) => return json!({
                "success": false,
                "error": format!("Invalid date format: {}. Expected YYYY-MM-DD", e)
            }).to_string(),
        };

        let natal_chart = self.storage.get_default_chart();

        let julian_day = date_to_julian_day(parsed_date);
        let positions = match calc_all_planets(julian_day) {
            Ok(p) => p,
            Err(e) => return json!({
                "success": false,
                "error": format!("Failed to calculate positions: {}", e)
            }).to_string(),
        };

        let mut transits = Vec::new();

        for (planet, position) in positions {
            let mut aspects_to_natal = Vec::new();

            if let Some(ref chart) = natal_chart {
                for (natal_planet, natal_pos) in &chart.planets {
                    if let Some((aspect_type, orb)) =
                        find_aspect(position.longitude, natal_pos.longitude, false)
                    {
                        aspects_to_natal.push(Aspect::new(
                            natal_planet.to_string(),
                            aspect_type,
                            (orb * 10.0).round() / 10.0,
                        ));
                    }
                }
            }

            let zodiac_pos = position.to_zodiac_position();

            transits.push(Transit {
                planet: planet.to_string(),
                sign: zodiac_pos.sign,
                degree: (zodiac_pos.degree * 10.0).round() / 10.0,
                retrograde: position.is_retrograde,
                aspects_to_natal,
            });
        }

        let response = GetDailyTransitsResponse {
            date: input.date,
            transits,
        };

        serde_json::to_string_pretty(&response).unwrap()
    }

    fn get_retrograde_status(&self, input: RetrogradeStatusInput) -> String {
        let parsed_date = match NaiveDate::parse_from_str(&input.date, "%Y-%m-%d") {
            Ok(d) => d,
            Err(e) => return json!({
                "success": false,
                "error": format!("Invalid date format: {}. Expected YYYY-MM-DD", e)
            }).to_string(),
        };

        let julian_day = date_to_julian_day(parsed_date);
        let include_upcoming = input.include_upcoming.unwrap_or(true);
        let days_ahead = input.days_ahead.unwrap_or(90);

        let mut currently_retrograde = Vec::new();
        let mut upcoming_retrogrades = Vec::new();

        for planet in Planet::all() {
            if !planet.can_retrograde() {
                continue;
            }

            let position = match calc_planet_position(*planet, julian_day) {
                Ok(p) => p,
                Err(e) => return json!({
                    "success": false,
                    "error": format!("Failed to calculate position: {}", e)
                }).to_string(),
            };

            if position.is_retrograde {
                let end_date = if let Ok(Some((jd, _))) =
                    find_next_station(*planet, julian_day, days_ahead as i32)
                {
                    Some(julian_day_to_date(jd).format("%Y-%m-%d").to_string())
                } else {
                    None
                };

                currently_retrograde.push(RetrogradeInfo {
                    planet: planet.to_string(),
                    retrograde: true,
                    retrograde_start: None,
                    retrograde_end: end_date.clone(),
                    direct_station: end_date,
                });
            } else if include_upcoming {
                if let Ok(Some((jd, is_turning_retrograde))) =
                    find_next_station(*planet, julian_day, days_ahead as i32)
                {
                    if is_turning_retrograde {
                        let days_until = (jd - julian_day).round() as i64;
                        let end_jd = if let Ok(Some((end, _))) =
                            find_next_station(*planet, jd + 1.0, 120)
                        {
                            end
                        } else {
                            jd + 21.0
                        };

                        upcoming_retrogrades.push(UpcomingRetrograde {
                            planet: planet.to_string(),
                            retrograde_start: julian_day_to_date(jd).format("%Y-%m-%d").to_string(),
                            retrograde_end: julian_day_to_date(end_jd).format("%Y-%m-%d").to_string(),
                            days_until,
                        });
                    }
                }
            }
        }

        let response = GetRetrogradeStatusResponse {
            date: input.date,
            currently_retrograde,
            upcoming_retrogrades,
        };

        serde_json::to_string_pretty(&response).unwrap()
    }

    fn get_lunar_info(&self, input: LunarInfoInput) -> String {
        let parsed_date = match NaiveDate::parse_from_str(&input.date, "%Y-%m-%d") {
            Ok(d) => d,
            Err(e) => return json!({
                "success": false,
                "error": format!("Invalid date format: {}. Expected YYYY-MM-DD", e)
            }).to_string(),
        };

        let julian_day = date_to_julian_day(parsed_date);

        let moon_pos = match calc_planet_position(Planet::Moon, julian_day) {
            Ok(p) => p,
            Err(e) => return json!({
                "success": false,
                "error": format!("Failed to calculate Moon position: {}", e)
            }).to_string(),
        };

        let phase_angle = match calc_sun_moon_angle(julian_day) {
            Ok(a) => a,
            Err(e) => return json!({
                "success": false,
                "error": format!("Failed to calculate phase angle: {}", e)
            }).to_string(),
        };

        let moon_zodiac = moon_pos.to_zodiac_position();

        let lunar_phase = LunarPhase {
            phase_name: LunarPhaseName::from_phase_angle(phase_angle),
            phase_percent: ((phase_angle / 360.0) * 100.0).round() as u8,
            illumination: (LunarPhaseName::illumination_from_angle(phase_angle) * 100.0).round() / 100.0,
            moon_sign: moon_zodiac.sign,
            moon_degree: (moon_zodiac.degree * 10.0).round() / 10.0,
        };

        let prev_new = find_previous_lunar_phase(julian_day, 0.0, 30).unwrap_or(julian_day - 14.0);
        let prev_full = find_previous_lunar_phase(julian_day, 180.0, 30).unwrap_or(julian_day - 7.0);
        let next_new = find_next_new_moon(julian_day, 30).ok().flatten().unwrap_or(julian_day + 29.5);
        let next_full = find_next_full_moon(julian_day, 30).ok().flatten().unwrap_or(julian_day + 14.0);

        let lunar_cycle = LunarCycle {
            new_moon: julian_day_to_date(prev_new).format("%Y-%m-%d").to_string(),
            full_moon: julian_day_to_date(prev_full).format("%Y-%m-%d").to_string(),
            next_new_moon: julian_day_to_date(next_new).format("%Y-%m-%d").to_string(),
            next_full_moon: julian_day_to_date(next_full).format("%Y-%m-%d").to_string(),
        };

        let void_of_course = VoidOfCourse {
            is_void: false,
            last_aspect_time: None,
            next_aspect_time: None,
            enters_void_at: None,
            exits_void_at: None,
        };

        let response = GetLunarInfoResponse {
            date: input.date,
            lunar_phase,
            void_of_course,
            lunar_cycle,
        };

        serde_json::to_string_pretty(&response).unwrap()
    }

    fn get_transit_report(&self, input: TransitReportInput) -> String {
        let parsed_start = match NaiveDate::parse_from_str(&input.start_date, "%Y-%m-%d") {
            Ok(d) => d,
            Err(e) => return json!({
                "success": false,
                "error": format!("Invalid start_date format: {}. Expected YYYY-MM-DD", e)
            }).to_string(),
        };

        let parsed_end = match NaiveDate::parse_from_str(&input.end_date, "%Y-%m-%d") {
            Ok(d) => d,
            Err(e) => return json!({
                "success": false,
                "error": format!("Invalid end_date format: {}. Expected YYYY-MM-DD", e)
            }).to_string(),
        };

        let include_minor = input.include_minor_aspects.unwrap_or(false);
        let natal_chart = self.storage.get_default_chart();

        let start_jd = date_to_julian_day(parsed_start);
        let end_jd = date_to_julian_day(parsed_end);
        let days = (end_jd - start_jd).ceil() as i32;

        let mut major_events = Vec::new();
        let mut lunar_events = Vec::new();
        let mut retrograde_events = Vec::new();

        for planet in Planet::all() {
            if let Ok(Some((jd, sign))) = find_next_sign_ingress(*planet, start_jd, days) {
                if jd <= end_jd {
                    let event_desc = if *planet == Planet::Sun {
                        let special = match sign {
                            crate::models::ZodiacSign::Aries => Some("Spring Equinox"),
                            crate::models::ZodiacSign::Cancer => Some("Summer Solstice"),
                            crate::models::ZodiacSign::Libra => Some("Fall Equinox"),
                            crate::models::ZodiacSign::Capricorn => Some("Winter Solstice"),
                            _ => None,
                        };
                        if let Some(special_name) = special {
                            format!("{} enters {} ({})", planet, sign, special_name)
                        } else {
                            format!("{} enters {}", planet, sign)
                        }
                    } else {
                        format!("{} enters {}", planet, sign)
                    };

                    major_events.push(MajorEvent {
                        date: julian_day_to_date(jd).format("%Y-%m-%d").to_string(),
                        event: event_desc,
                        event_type: "sign_change".to_string(),
                        orb: None,
                        affected_planets: vec![planet.to_string()],
                    });
                }
            }

            if planet.can_retrograde() {
                if let Ok(Some((jd, is_retrograde))) = find_next_station(*planet, start_jd, days) {
                    if jd <= end_jd {
                        let event_desc = if is_retrograde {
                            format!("{} stations retrograde", planet)
                        } else {
                            format!("{} stations direct", planet)
                        };

                        retrograde_events.push(MajorEvent {
                            date: julian_day_to_date(jd).format("%Y-%m-%d").to_string(),
                            event: event_desc,
                            event_type: "station".to_string(),
                            orb: None,
                            affected_planets: vec![planet.to_string()],
                        });
                    }
                }
            }
        }

        if let Some(ref chart) = natal_chart {
            let mut jd = start_jd;
            while jd <= end_jd {
                let positions = calc_all_planets(jd).unwrap_or_default();

                for (transit_planet, transit_pos) in &positions {
                    for (natal_planet, natal_pos) in &chart.planets {
                        if let Some((aspect_type, orb)) =
                            find_aspect(transit_pos.longitude, natal_pos.longitude, include_minor)
                        {
                            if orb < 0.5 {
                                major_events.push(MajorEvent {
                                    date: julian_day_to_date(jd).format("%Y-%m-%d").to_string(),
                                    event: format!("{} {} natal {}", transit_planet, aspect_type, natal_planet),
                                    event_type: "aspect".to_string(),
                                    orb: Some((orb * 10.0).round() / 10.0),
                                    affected_planets: vec![transit_planet.to_string(), natal_planet.to_string()],
                                });
                            }
                        }
                    }
                }

                jd += 1.0;
            }
        }

        let mut jd = start_jd;
        while jd <= end_jd {
            if let Ok(Some(new_jd)) = find_next_new_moon(jd, (end_jd - jd + 1.0) as i32) {
                if new_jd <= end_jd {
                    lunar_events.push(LunarEvent {
                        date: julian_day_to_date(new_jd).format("%Y-%m-%d").to_string(),
                        event: "New Moon".to_string(),
                        event_type: "lunar_phase".to_string(),
                    });
                    jd = new_jd + 1.0;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let mut jd = start_jd;
        while jd <= end_jd {
            if let Ok(Some(full_jd)) = find_next_full_moon(jd, (end_jd - jd + 1.0) as i32) {
                if full_jd <= end_jd {
                    lunar_events.push(LunarEvent {
                        date: julian_day_to_date(full_jd).format("%Y-%m-%d").to_string(),
                        event: "Full Moon".to_string(),
                        event_type: "lunar_phase".to_string(),
                    });
                    jd = full_jd + 1.0;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        major_events.sort_by(|a, b| a.date.cmp(&b.date));
        lunar_events.sort_by(|a, b| a.date.cmp(&b.date));
        retrograde_events.sort_by(|a, b| a.date.cmp(&b.date));

        let response = GetTransitReportResponse {
            period: DateRange {
                start_date: input.start_date,
                end_date: input.end_date,
            },
            major_events,
            lunar_events,
            retrograde_events,
        };

        serde_json::to_string_pretty(&response).unwrap()
    }

    fn list_natal_charts(&self) -> String {
        let charts = self.storage.list_charts();

        let response = json!({
            "charts": charts,
            "count": charts.len()
        });

        serde_json::to_string_pretty(&response).unwrap()
    }

    fn search_natal_charts(&self, input: SearchNatalChartsInput) -> String {
        let charts = self.storage.search_charts(&input.query);

        let response = json!({
            "query": input.query,
            "results": charts,
            "count": charts.len()
        });

        serde_json::to_string_pretty(&response).unwrap()
    }

    fn get_natal_chart(&self, input: GetNatalChartInput) -> String {
        let chart = match self.storage.get_chart(&input.name) {
            Some(c) => c,
            None => return json!({
                "success": false,
                "error": format!("Natal chart '{}' not found", input.name)
            }).to_string(),
        };

        let summary = NatalChartSummary::from(&chart);
        let response = json!({
            "name": chart.name,
            "birth_date": chart.birth_date,
            "birth_time": chart.birth_time,
            "birth_location": chart.birth_location,
            "positions": summary
        });

        serde_json::to_string_pretty(&response).unwrap()
    }

    fn delete_natal_chart(&self, input: DeleteNatalChartInput) -> String {
        // Verify the chart exists with exact name and birth_date match
        let chart = match self.storage.get_chart_exact(&input.name, &input.birth_date) {
            Some(c) => c,
            None => {
                // Check if a chart with this name exists but different birth date
                if let Some(existing) = self.storage.get_chart(&input.name) {
                    return json!({
                        "success": false,
                        "error": format!(
                            "Chart '{}' exists but with birth date '{}'. You provided '{}'. Please use the correct birth date to delete.",
                            input.name, existing.birth_date, input.birth_date
                        )
                    }).to_string();
                }
                return json!({
                    "success": false,
                    "error": format!("Natal chart '{}' not found", input.name)
                }).to_string();
            }
        };

        // Delete the chart using exact key
        match self.storage.delete_chart_exact(&input.name, &input.birth_date) {
            Ok(true) => json!({
                "success": true,
                "message": format!("Natal chart '{}' (born {}) has been deleted", chart.name, chart.birth_date)
            }).to_string(),
            Ok(false) => json!({
                "success": false,
                "error": format!("Natal chart '{}' not found", input.name)
            }).to_string(),
            Err(e) => json!({
                "success": false,
                "error": format!("Failed to delete chart: {}", e)
            }).to_string(),
        }
    }

    fn get_compatibility(&self, input: GetCompatibilityInput) -> String {
        // Load both charts
        let chart1 = match self.storage.get_chart(&input.person1_name) {
            Some(c) => c,
            None => return json!({
                "success": false,
                "error": format!("Natal chart '{}' not found", input.person1_name)
            }).to_string(),
        };

        let chart2 = match self.storage.get_chart(&input.person2_name) {
            Some(c) => c,
            None => return json!({
                "success": false,
                "error": format!("Natal chart '{}' not found", input.person2_name)
            }).to_string(),
        };

        let include_minor = input.include_minor_aspects.unwrap_or(false);
        let mut synastry_aspects = Vec::new();
        let mut exact_aspects = Vec::new();

        // Compare every planet in chart1 against every planet in chart2
        for planet1 in Planet::all() {
            let pos1 = match chart1.planets.get(planet1) {
                Some(p) => p,
                None => continue,
            };

            let house1 = chart1.planet_positions.get(planet1).map(|p| p.house);

            for planet2 in Planet::all() {
                let pos2 = match chart2.planets.get(planet2) {
                    Some(p) => p,
                    None => continue,
                };

                let house2 = chart2.planet_positions.get(planet2).map(|p| p.house);

                // Check for aspects
                if let Some((aspect_type, orb)) = find_aspect(pos1.longitude, pos2.longitude, include_minor) {
                    let is_exact = orb < 1.0;

                    let aspect_info = json!({
                        "person1_planet": planet1.to_string(),
                        "person1_position": pos1.format_degree_sign(),
                        "person1_house": house1,
                        "person2_planet": planet2.to_string(),
                        "person2_position": pos2.format_degree_sign(),
                        "person2_house": house2,
                        "aspect": aspect_type.to_string(),
                        "orb": (orb * 100.0).round() / 100.0,
                        "is_exact": is_exact,
                        "is_major": aspect_type.is_major()
                    });

                    if is_exact {
                        exact_aspects.push(json!({
                            "aspect": format!("{} {} {} ({})",
                                input.person1_name, planet1,
                                aspect_type,
                                planet2),
                            "description": format!("{}'s {} {} {}'s {} (orb: {:.2}°)",
                                input.person1_name, planet1,
                                aspect_type,
                                input.person2_name, planet2,
                                orb)
                        }));
                    }

                    synastry_aspects.push(aspect_info);
                }
            }
        }

        // Build person summaries
        let person1_summary = json!({
            "name": chart1.name,
            "birth_date": chart1.birth_date,
            "sun": chart1.planets.get(&Planet::Sun).map(|p| p.format_degree_sign()),
            "moon": chart1.planets.get(&Planet::Moon).map(|p| p.format_degree_sign()),
            "ascendant": chart1.ascendant.as_ref().map(|p| p.format_degree_sign()),
            "venus": chart1.planets.get(&Planet::Venus).map(|p| p.format_degree_sign()),
            "mars": chart1.planets.get(&Planet::Mars).map(|p| p.format_degree_sign())
        });

        let person2_summary = json!({
            "name": chart2.name,
            "birth_date": chart2.birth_date,
            "sun": chart2.planets.get(&Planet::Sun).map(|p| p.format_degree_sign()),
            "moon": chart2.planets.get(&Planet::Moon).map(|p| p.format_degree_sign()),
            "ascendant": chart2.ascendant.as_ref().map(|p| p.format_degree_sign()),
            "venus": chart2.planets.get(&Planet::Venus).map(|p| p.format_degree_sign()),
            "mars": chart2.planets.get(&Planet::Mars).map(|p| p.format_degree_sign())
        });

        // Count aspect types for summary
        let mut conjunction_count = 0;
        let mut trine_count = 0;
        let mut sextile_count = 0;
        let mut square_count = 0;
        let mut opposition_count = 0;

        for aspect in &synastry_aspects {
            match aspect.get("aspect").and_then(|v| v.as_str()) {
                Some("conjunction") => conjunction_count += 1,
                Some("trine") => trine_count += 1,
                Some("sextile") => sextile_count += 1,
                Some("square") => square_count += 1,
                Some("opposition") => opposition_count += 1,
                _ => {}
            }
        }

        let response = json!({
            "success": true,
            "person1": person1_summary,
            "person2": person2_summary,
            "aspects": synastry_aspects,
            "summary": {
                "total_aspects": synastry_aspects.len(),
                "exact_aspects_count": exact_aspects.len(),
                "exact_aspects": exact_aspects,
                "aspect_counts": {
                    "conjunctions": conjunction_count,
                    "trines": trine_count,
                    "sextiles": sextile_count,
                    "squares": square_count,
                    "oppositions": opposition_count
                },
                "harmonious_aspects": trine_count + sextile_count + conjunction_count,
                "challenging_aspects": square_count + opposition_count
            }
        });

        serde_json::to_string_pretty(&response).unwrap()
    }

    fn get_tools(&self) -> Vec<Tool> {
        vec![
            Tool::new(
                "store_natal_chart",
                "Store a natal chart with birth data for future transit calculations. The chart will be saved permanently and used for aspect calculations.",
                schema_to_value::<StoreNatalChartInput>(),
            ),
            Tool::new(
                "get_daily_transits",
                "Get current planetary positions and their aspects to your natal chart for a specific date.",
                schema_to_value::<DailyTransitsInput>(),
            ),
            Tool::new(
                "get_retrograde_status",
                "Get which planets are currently retrograde and upcoming retrograde periods.",
                schema_to_value::<RetrogradeStatusInput>(),
            ),
            Tool::new(
                "get_lunar_info",
                "Get current lunar phase, void-of-course status, and lunar cycle dates.",
                schema_to_value::<LunarInfoInput>(),
            ),
            Tool::new(
                "get_transit_report",
                "Get a summary of major astrological events over a specified date range including sign changes, aspects to natal chart, and lunar events.",
                schema_to_value::<TransitReportInput>(),
            ),
            Tool::new(
                "list_natal_charts",
                "List all stored natal charts by name.",
                empty_schema(),
            ),
            Tool::new(
                "search_natal_charts",
                "Search for natal charts by name (case-insensitive partial match).",
                schema_to_value::<SearchNatalChartsInput>(),
            ),
            Tool::new(
                "get_natal_chart",
                "Get a stored natal chart by name.",
                schema_to_value::<GetNatalChartInput>(),
            ),
            Tool::new(
                "delete_natal_chart",
                "Delete a stored natal chart. Requires both name and birth date for confirmation.",
                schema_to_value::<DeleteNatalChartInput>(),
            ),
            Tool::new(
                "get_compatibility",
                "Analyze synastry/compatibility between two natal charts. Compares all planetary aspects between two people.",
                schema_to_value::<GetCompatibilityInput>(),
            ),
        ]
    }
}

impl ServerHandler for StelliumServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "Stellium - MCP Server providing ephemeris data and astrological calculations. \
                 Store your natal chart first with store_natal_chart, then use get_daily_transits \
                 to see how current planetary positions aspect your chart."
                    .to_string(),
            ),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, rmcp::ErrorData> {
        Ok(ListToolsResult {
            tools: self.get_tools(),
            next_cursor: None,
            meta: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let args: Value = Value::Object(request.arguments.clone().unwrap_or_default());

        let result = match request.name.as_ref() {
            "store_natal_chart" => {
                let input: StoreNatalChartInput = serde_json::from_value(args)
                    .map_err(|e| rmcp::ErrorData::invalid_params(e.to_string(), None))?;
                self.store_natal_chart(input)
            }
            "get_daily_transits" => {
                let input: DailyTransitsInput = serde_json::from_value(args)
                    .map_err(|e| rmcp::ErrorData::invalid_params(e.to_string(), None))?;
                self.get_daily_transits(input)
            }
            "get_retrograde_status" => {
                let input: RetrogradeStatusInput = serde_json::from_value(args)
                    .map_err(|e| rmcp::ErrorData::invalid_params(e.to_string(), None))?;
                self.get_retrograde_status(input)
            }
            "get_lunar_info" => {
                let input: LunarInfoInput = serde_json::from_value(args)
                    .map_err(|e| rmcp::ErrorData::invalid_params(e.to_string(), None))?;
                self.get_lunar_info(input)
            }
            "get_transit_report" => {
                let input: TransitReportInput = serde_json::from_value(args)
                    .map_err(|e| rmcp::ErrorData::invalid_params(e.to_string(), None))?;
                self.get_transit_report(input)
            }
            "list_natal_charts" => self.list_natal_charts(),
            "search_natal_charts" => {
                let input: SearchNatalChartsInput = serde_json::from_value(args)
                    .map_err(|e| rmcp::ErrorData::invalid_params(e.to_string(), None))?;
                self.search_natal_charts(input)
            }
            "get_natal_chart" => {
                let input: GetNatalChartInput = serde_json::from_value(args)
                    .map_err(|e| rmcp::ErrorData::invalid_params(e.to_string(), None))?;
                self.get_natal_chart(input)
            }
            "delete_natal_chart" => {
                let input: DeleteNatalChartInput = serde_json::from_value(args)
                    .map_err(|e| rmcp::ErrorData::invalid_params(e.to_string(), None))?;
                self.delete_natal_chart(input)
            }
            "get_compatibility" => {
                let input: GetCompatibilityInput = serde_json::from_value(args)
                    .map_err(|e| rmcp::ErrorData::invalid_params(e.to_string(), None))?;
                self.get_compatibility(input)
            }
            _ => {
                return Err(rmcp::ErrorData::invalid_params(
                    format!("Unknown tool: {}", request.name),
                    None,
                ))
            }
        };

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
}

/// Find previous lunar phase (search backwards)
fn find_previous_lunar_phase(start_jd: f64, target_angle: f64, max_days: i32) -> Option<f64> {
    let step = 0.5;
    let mut jd = start_jd;
    let end_jd = start_jd - max_days as f64;

    let mut prev_angle = calc_sun_moon_angle(jd).ok()?;

    while jd > end_jd {
        jd -= step;
        let current_angle = calc_sun_moon_angle(jd).ok()?;

        let crossed = if target_angle < 10.0 || target_angle > 350.0 {
            current_angle > 350.0 && prev_angle < 10.0
        } else {
            current_angle > target_angle && prev_angle <= target_angle
        };

        if crossed {
            return Some(jd);
        }

        prev_angle = current_angle;
    }

    None
}
