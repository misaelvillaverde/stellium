use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{Planet, ZodiacPosition};

/// Request to store a natal chart
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct StoreNatalChartRequest {
    /// Name of the person
    #[schemars(description = "Name of the person for this natal chart")]
    pub name: String,

    /// Birth date in YYYY-MM-DD format
    #[schemars(description = "Birth date in YYYY-MM-DD format")]
    pub birth_date: String,

    /// Birth time in HH:MM:SS format
    #[schemars(description = "Birth time in HH:MM:SS format")]
    pub birth_time: String,

    /// Birth location name
    #[schemars(description = "Birth location name (e.g., 'Panama City, Panama')")]
    pub birth_location: String,

    /// Latitude of birth location
    #[schemars(description = "Latitude of birth location in decimal degrees")]
    pub latitude: f64,

    /// Longitude of birth location
    #[schemars(description = "Longitude of birth location in decimal degrees")]
    pub longitude: f64,

    /// Timezone identifier (e.g., "America/Panama")
    #[schemars(description = "Timezone identifier (e.g., 'America/Panama')")]
    pub timezone: String,
}

impl StoreNatalChartRequest {
    pub fn parse_date(&self) -> Result<NaiveDate, chrono::ParseError> {
        NaiveDate::parse_from_str(&self.birth_date, "%Y-%m-%d")
    }

    pub fn parse_time(&self) -> Result<NaiveTime, chrono::ParseError> {
        NaiveTime::parse_from_str(&self.birth_time, "%H:%M:%S")
    }
}

/// House cusp data for a natal chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HouseCusps {
    /// All 12 house cusps (index 0 = 1st house cusp, etc.)
    pub cusps: Vec<ZodiacPosition>,
    /// House system used (e.g., "Placidus", "Whole Sign")
    pub system: String,
}

/// Planet position with house placement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetPosition {
    pub position: ZodiacPosition,
    /// Which house this planet is in (1-12)
    pub house: u8,
    /// Whether the planet is retrograde
    pub is_retrograde: bool,
}

/// Stored natal chart with calculated positions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatalChart {
    pub name: String,
    pub birth_date: String,
    pub birth_time: String,
    pub birth_location: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,

    /// Planetary positions at birth (legacy format for compatibility)
    pub planets: HashMap<Planet, ZodiacPosition>,

    /// Planetary positions with house placements
    #[serde(default)]
    pub planet_positions: HashMap<Planet, PlanetPosition>,

    /// Ascendant (rising sign)
    pub ascendant: Option<ZodiacPosition>,

    /// Midheaven (MC)
    pub midheaven: Option<ZodiacPosition>,

    /// Vertex point
    #[serde(default)]
    pub vertex: Option<ZodiacPosition>,

    /// All 12 house cusps
    #[serde(default)]
    pub houses: Option<HouseCusps>,
}

impl NatalChart {
    pub fn new(request: &StoreNatalChartRequest) -> Self {
        Self {
            name: request.name.clone(),
            birth_date: request.birth_date.clone(),
            birth_time: request.birth_time.clone(),
            birth_location: request.birth_location.clone(),
            latitude: request.latitude,
            longitude: request.longitude,
            timezone: request.timezone.clone(),
            planets: HashMap::new(),
            planet_positions: HashMap::new(),
            ascendant: None,
            midheaven: None,
            vertex: None,
            houses: None,
        }
    }

    /// Get position for a planet
    pub fn get_planet_position(&self, planet: &Planet) -> Option<&ZodiacPosition> {
        self.planets.get(planet)
    }

    /// Get house number (1-12) for a planet
    pub fn get_planet_house(&self, planet: &Planet) -> Option<u8> {
        self.planet_positions.get(planet).map(|p| p.house)
    }

    /// Get house cusp position by house number (1-12)
    pub fn get_house_cusp(&self, house_num: u8) -> Option<&ZodiacPosition> {
        if house_num < 1 || house_num > 12 {
            return None;
        }
        self.houses.as_ref().and_then(|h| h.cusps.get((house_num - 1) as usize))
    }
}

/// Response after storing a natal chart
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct StoreNatalChartResponse {
    pub success: bool,
    pub message: String,
    pub natal_chart: NatalChartSummary,
}

/// Detailed planet position for summary output
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct PlanetSummary {
    /// Position as "X° Sign" format
    pub position: String,
    /// House number (1-12)
    pub house: Option<u8>,
    /// Whether retrograde
    pub retrograde: bool,
}

/// House cusp summary
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct HouseSummary {
    /// House number (1-12)
    pub house: u8,
    /// Cusp position as "X° Sign" format
    pub cusp: String,
}

/// Summary of natal chart positions for display
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct NatalChartSummary {
    pub sun: PlanetSummary,
    pub moon: PlanetSummary,
    pub mercury: PlanetSummary,
    pub venus: PlanetSummary,
    pub mars: PlanetSummary,
    pub jupiter: PlanetSummary,
    pub saturn: PlanetSummary,
    pub uranus: PlanetSummary,
    pub neptune: PlanetSummary,
    pub pluto: PlanetSummary,
    pub north_node: PlanetSummary,
    pub ascendant: String,
    pub midheaven: String,
    pub houses: Vec<HouseSummary>,
}

impl From<&NatalChart> for NatalChartSummary {
    fn from(chart: &NatalChart) -> Self {
        let get_planet_summary = |planet: Planet| -> PlanetSummary {
            let position = chart
                .planets
                .get(&planet)
                .map(|p| p.format_degree_sign())
                .unwrap_or_else(|| "Unknown".to_string());

            let (house, retrograde) = chart
                .planet_positions
                .get(&planet)
                .map(|p| (Some(p.house), p.is_retrograde))
                .unwrap_or((None, false));

            PlanetSummary {
                position,
                house,
                retrograde,
            }
        };

        let houses: Vec<HouseSummary> = chart
            .houses
            .as_ref()
            .map(|h| {
                h.cusps
                    .iter()
                    .enumerate()
                    .map(|(i, cusp)| HouseSummary {
                        house: (i + 1) as u8,
                        cusp: cusp.format_degree_sign(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        Self {
            sun: get_planet_summary(Planet::Sun),
            moon: get_planet_summary(Planet::Moon),
            mercury: get_planet_summary(Planet::Mercury),
            venus: get_planet_summary(Planet::Venus),
            mars: get_planet_summary(Planet::Mars),
            jupiter: get_planet_summary(Planet::Jupiter),
            saturn: get_planet_summary(Planet::Saturn),
            uranus: get_planet_summary(Planet::Uranus),
            neptune: get_planet_summary(Planet::Neptune),
            pluto: get_planet_summary(Planet::Pluto),
            north_node: get_planet_summary(Planet::NorthNode),
            ascendant: chart
                .ascendant
                .as_ref()
                .map(|p| p.format_degree_sign())
                .unwrap_or_else(|| "Unknown".to_string()),
            midheaven: chart
                .midheaven
                .as_ref()
                .map(|p| p.format_degree_sign())
                .unwrap_or_else(|| "Unknown".to_string()),
            houses,
        }
    }
}
