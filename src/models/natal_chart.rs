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

    /// Planetary positions at birth
    pub planets: HashMap<Planet, ZodiacPosition>,

    /// Ascendant (rising sign)
    pub ascendant: Option<ZodiacPosition>,

    /// Midheaven (MC)
    pub midheaven: Option<ZodiacPosition>,
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
            ascendant: None,
            midheaven: None,
        }
    }

    /// Get position for a planet
    pub fn get_planet_position(&self, planet: &Planet) -> Option<&ZodiacPosition> {
        self.planets.get(planet)
    }
}

/// Response after storing a natal chart
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct StoreNatalChartResponse {
    pub success: bool,
    pub message: String,
    pub natal_chart: NatalChartSummary,
}

/// Summary of natal chart positions for display
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct NatalChartSummary {
    pub sun: String,
    pub moon: String,
    pub ascendant: String,
    pub mercury: String,
    pub venus: String,
    pub mars: String,
    pub jupiter: String,
    pub saturn: String,
    pub uranus: String,
    pub neptune: String,
    pub pluto: String,
}

impl From<&NatalChart> for NatalChartSummary {
    fn from(chart: &NatalChart) -> Self {
        let get_pos = |planet: Planet| -> String {
            chart
                .planets
                .get(&planet)
                .map(|p| p.format_degree_sign())
                .unwrap_or_else(|| "Unknown".to_string())
        };

        Self {
            sun: get_pos(Planet::Sun),
            moon: get_pos(Planet::Moon),
            ascendant: chart
                .ascendant
                .as_ref()
                .map(|p| p.format_degree_sign())
                .unwrap_or_else(|| "Unknown".to_string()),
            mercury: get_pos(Planet::Mercury),
            venus: get_pos(Planet::Venus),
            mars: get_pos(Planet::Mars),
            jupiter: get_pos(Planet::Jupiter),
            saturn: get_pos(Planet::Saturn),
            uranus: get_pos(Planet::Uranus),
            neptune: get_pos(Planet::Neptune),
            pluto: get_pos(Planet::Pluto),
        }
    }
}
