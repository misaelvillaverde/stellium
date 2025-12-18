use serde::{Deserialize, Serialize};
use std::fmt;

/// Types of astrological aspects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AspectType {
    Conjunction,
    Sextile,
    Square,
    Trine,
    Opposition,
    // Minor aspects (optional)
    SemiSextile,
    Quincunx,
    SemiSquare,
    Sesquiquadrate,
}

impl AspectType {
    /// Get the exact angle for this aspect
    pub fn angle(&self) -> f64 {
        match self {
            AspectType::Conjunction => 0.0,
            AspectType::SemiSextile => 30.0,
            AspectType::SemiSquare => 45.0,
            AspectType::Sextile => 60.0,
            AspectType::Square => 90.0,
            AspectType::Trine => 120.0,
            AspectType::Sesquiquadrate => 135.0,
            AspectType::Quincunx => 150.0,
            AspectType::Opposition => 180.0,
        }
    }

    /// Get the default orb (tolerance) for this aspect
    pub fn default_orb(&self) -> f64 {
        match self {
            AspectType::Conjunction => 8.0,
            AspectType::Opposition => 8.0,
            AspectType::Trine => 8.0,
            AspectType::Square => 7.0,
            AspectType::Sextile => 6.0,
            AspectType::SemiSextile => 2.0,
            AspectType::Quincunx => 3.0,
            AspectType::SemiSquare => 2.0,
            AspectType::Sesquiquadrate => 2.0,
        }
    }

    /// Whether this is a major aspect
    pub fn is_major(&self) -> bool {
        matches!(
            self,
            AspectType::Conjunction
                | AspectType::Sextile
                | AspectType::Square
                | AspectType::Trine
                | AspectType::Opposition
        )
    }

    /// Get all major aspects
    pub fn major_aspects() -> &'static [AspectType] {
        &[
            AspectType::Conjunction,
            AspectType::Sextile,
            AspectType::Square,
            AspectType::Trine,
            AspectType::Opposition,
        ]
    }

    /// Get all aspects including minor ones
    pub fn all_aspects() -> &'static [AspectType] {
        &[
            AspectType::Conjunction,
            AspectType::SemiSextile,
            AspectType::SemiSquare,
            AspectType::Sextile,
            AspectType::Square,
            AspectType::Trine,
            AspectType::Sesquiquadrate,
            AspectType::Quincunx,
            AspectType::Opposition,
        ]
    }
}

impl fmt::Display for AspectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            AspectType::Conjunction => "conjunction",
            AspectType::SemiSextile => "semi-sextile",
            AspectType::SemiSquare => "semi-square",
            AspectType::Sextile => "sextile",
            AspectType::Square => "square",
            AspectType::Trine => "trine",
            AspectType::Sesquiquadrate => "sesquiquadrate",
            AspectType::Quincunx => "quincunx",
            AspectType::Opposition => "opposition",
        };
        write!(f, "{}", name)
    }
}

/// Check if two positions form an aspect
pub fn find_aspect(
    longitude1: f64,
    longitude2: f64,
    include_minor: bool,
) -> Option<(AspectType, f64)> {
    let aspects = if include_minor {
        AspectType::all_aspects()
    } else {
        AspectType::major_aspects()
    };

    // Calculate the shortest angular distance
    let diff = (longitude1 - longitude2).abs();
    let angular_distance = if diff > 180.0 { 360.0 - diff } else { diff };

    for aspect in aspects {
        let target_angle = aspect.angle();
        let orb = (angular_distance - target_angle).abs();

        if orb <= aspect.default_orb() {
            return Some((*aspect, orb));
        }
    }

    None
}

/// An aspect between two celestial bodies
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Aspect {
    pub natal_planet: String,
    pub aspect_type: AspectType,
    /// How close the aspect is to exact (in degrees)
    pub orb: f64,
    /// Whether the aspect is within 1 degree of exact
    pub is_exact: bool,
}

impl Aspect {
    pub fn new(natal_planet: String, aspect_type: AspectType, orb: f64) -> Self {
        Self {
            natal_planet,
            aspect_type,
            is_exact: orb < 1.0,
            orb,
        }
    }
}
