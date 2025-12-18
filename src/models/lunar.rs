use serde::{Deserialize, Serialize};

use super::ZodiacSign;

/// Request to get lunar information
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct GetLunarInfoRequest {
    /// Date in YYYY-MM-DD format
    #[schemars(description = "Date to get lunar information for in YYYY-MM-DD format")]
    pub date: String,
}

/// Lunar phase names
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LunarPhaseName {
    NewMoon,
    WaxingCrescent,
    FirstQuarter,
    WaxingGibbous,
    FullMoon,
    WaningGibbous,
    LastQuarter,
    WaningCrescent,
}

impl LunarPhaseName {
    /// Determine phase from the angle between Sun and Moon
    pub fn from_phase_angle(angle: f64) -> Self {
        // Phase angle: Moon longitude - Sun longitude (normalized 0-360)
        let normalized = angle.rem_euclid(360.0);

        match normalized {
            a if a < 22.5 => LunarPhaseName::NewMoon,
            a if a < 67.5 => LunarPhaseName::WaxingCrescent,
            a if a < 112.5 => LunarPhaseName::FirstQuarter,
            a if a < 157.5 => LunarPhaseName::WaxingGibbous,
            a if a < 202.5 => LunarPhaseName::FullMoon,
            a if a < 247.5 => LunarPhaseName::WaningGibbous,
            a if a < 292.5 => LunarPhaseName::LastQuarter,
            a if a < 337.5 => LunarPhaseName::WaningCrescent,
            _ => LunarPhaseName::NewMoon,
        }
    }

    /// Calculate illumination percentage from phase angle
    pub fn illumination_from_angle(angle: f64) -> f64 {
        // Illumination ranges from 0 (new) to 1 (full)
        // Use cosine function: illumination = (1 - cos(angle)) / 2
        let normalized = angle.rem_euclid(360.0);
        let radians = normalized.to_radians();
        (1.0 - radians.cos()) / 2.0
    }
}

/// Current lunar phase information
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct LunarPhase {
    pub phase_name: LunarPhaseName,
    /// Phase progress as percentage (0-100)
    pub phase_percent: u8,
    /// Illumination as decimal (0.0-1.0)
    pub illumination: f64,
    pub moon_sign: ZodiacSign,
    pub moon_degree: f64,
}

/// Void-of-course moon information
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct VoidOfCourse {
    pub is_void: bool,
    pub last_aspect_time: Option<String>,
    pub next_aspect_time: Option<String>,
    pub enters_void_at: Option<String>,
    pub exits_void_at: Option<String>,
}

/// Lunar cycle dates
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct LunarCycle {
    pub new_moon: String,
    pub full_moon: String,
    pub next_new_moon: String,
    pub next_full_moon: String,
}

/// Response for lunar information
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct GetLunarInfoResponse {
    pub date: String,
    pub lunar_phase: LunarPhase,
    pub void_of_course: VoidOfCourse,
    pub lunar_cycle: LunarCycle,
}
