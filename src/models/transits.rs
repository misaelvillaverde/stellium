use serde::{Deserialize, Serialize};

use super::{Aspect, ZodiacSign};

/// Request to get daily transits
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct GetDailyTransitsRequest {
    /// Date in YYYY-MM-DD format
    #[schemars(description = "Date to get transits for in YYYY-MM-DD format")]
    pub date: String,
}

/// A transit (current planetary position with aspects to natal chart)
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct Transit {
    pub planet: String,
    pub sign: ZodiacSign,
    pub degree: f64,
    pub retrograde: bool,
    pub aspects_to_natal: Vec<Aspect>,
}

/// Response for daily transits
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct GetDailyTransitsResponse {
    pub date: String,
    pub transits: Vec<Transit>,
}

/// Request to get retrograde status
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct GetRetrogradeStatusRequest {
    /// Date in YYYY-MM-DD format
    #[schemars(description = "Date to check retrograde status for in YYYY-MM-DD format")]
    pub date: String,

    /// Whether to include upcoming retrogrades
    #[schemars(description = "Whether to include upcoming retrograde periods")]
    pub include_upcoming: Option<bool>,

    /// Number of days to look ahead for upcoming retrogrades
    #[schemars(description = "Number of days to look ahead for upcoming retrogrades (default: 90)")]
    pub days_ahead: Option<i64>,
}

/// Information about a retrograde period
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct RetrogradeInfo {
    pub planet: String,
    pub retrograde: bool,
    pub retrograde_start: Option<String>,
    pub retrograde_end: Option<String>,
    pub direct_station: Option<String>,
}

/// Information about an upcoming retrograde
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct UpcomingRetrograde {
    pub planet: String,
    pub retrograde_start: String,
    pub retrograde_end: String,
    pub days_until: i64,
}

/// Response for retrograde status
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct GetRetrogradeStatusResponse {
    pub date: String,
    pub currently_retrograde: Vec<RetrogradeInfo>,
    pub upcoming_retrogrades: Vec<UpcomingRetrograde>,
}

/// Request to get transit report over a date range
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct GetTransitReportRequest {
    /// Start date in YYYY-MM-DD format
    #[schemars(description = "Start date in YYYY-MM-DD format")]
    pub start_date: String,

    /// End date in YYYY-MM-DD format
    #[schemars(description = "End date in YYYY-MM-DD format")]
    pub end_date: String,

    /// Whether to include minor aspects
    #[schemars(description = "Whether to include minor aspects (default: false)")]
    pub include_minor_aspects: Option<bool>,
}

/// A major astrological event
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct MajorEvent {
    pub date: String,
    pub event: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub orb: Option<f64>,
    pub affected_planets: Vec<String>,
}

/// A lunar event
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct LunarEvent {
    pub date: String,
    pub event: String,
    #[serde(rename = "type")]
    pub event_type: String,
}

/// Response for transit report
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct GetTransitReportResponse {
    pub period: DateRange,
    pub major_events: Vec<MajorEvent>,
    pub lunar_events: Vec<LunarEvent>,
    pub retrograde_events: Vec<MajorEvent>,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct DateRange {
    pub start_date: String,
    pub end_date: String,
}
