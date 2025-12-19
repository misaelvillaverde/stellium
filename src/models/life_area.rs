//! Life area mappings for astrological houses

use serde::{Deserialize, Serialize};
use std::fmt;

/// Life areas corresponding to astrological houses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LifeArea {
    /// 1st House - Self, identity, appearance, first impressions
    Identity,
    /// 2nd House - Money, possessions, values, self-worth
    Finances,
    /// 3rd House - Communication, siblings, short trips, learning
    Communication,
    /// 4th House - Home, family, roots, emotional foundation
    Home,
    /// 5th House - Creativity, romance, children, pleasure
    Romance,
    /// 6th House - Work, health, daily routines, service
    Work,
    /// 7th House - Partnerships, marriage, open enemies, contracts
    Partnerships,
    /// 8th House - Transformation, shared resources, intimacy, death/rebirth
    Transformation,
    /// 9th House - Higher education, travel, philosophy, spirituality
    Spirituality,
    /// 10th House - Career, public image, reputation, authority
    Career,
    /// 11th House - Friends, groups, hopes, social causes
    Community,
    /// 12th House - Subconscious, hidden matters, isolation, endings
    Subconscious,
}

impl LifeArea {
    /// Get the life area for a given house number (1-12)
    pub fn from_house(house: u8) -> Option<Self> {
        match house {
            1 => Some(LifeArea::Identity),
            2 => Some(LifeArea::Finances),
            3 => Some(LifeArea::Communication),
            4 => Some(LifeArea::Home),
            5 => Some(LifeArea::Romance),
            6 => Some(LifeArea::Work),
            7 => Some(LifeArea::Partnerships),
            8 => Some(LifeArea::Transformation),
            9 => Some(LifeArea::Spirituality),
            10 => Some(LifeArea::Career),
            11 => Some(LifeArea::Community),
            12 => Some(LifeArea::Subconscious),
            _ => None,
        }
    }

    /// Get the house number for this life area
    pub fn house_number(&self) -> u8 {
        match self {
            LifeArea::Identity => 1,
            LifeArea::Finances => 2,
            LifeArea::Communication => 3,
            LifeArea::Home => 4,
            LifeArea::Romance => 5,
            LifeArea::Work => 6,
            LifeArea::Partnerships => 7,
            LifeArea::Transformation => 8,
            LifeArea::Spirituality => 9,
            LifeArea::Career => 10,
            LifeArea::Community => 11,
            LifeArea::Subconscious => 12,
        }
    }

    /// Get a description of what this life area governs
    pub fn description(&self) -> &'static str {
        match self {
            LifeArea::Identity => "Self, identity, appearance, first impressions",
            LifeArea::Finances => "Money, possessions, values, self-worth",
            LifeArea::Communication => "Communication, siblings, short trips, learning",
            LifeArea::Home => "Home, family, roots, emotional foundation",
            LifeArea::Romance => "Creativity, romance, children, pleasure",
            LifeArea::Work => "Work, health, daily routines, service",
            LifeArea::Partnerships => "Partnerships, marriage, contracts",
            LifeArea::Transformation => "Transformation, shared resources, intimacy",
            LifeArea::Spirituality => "Higher education, travel, philosophy, beliefs",
            LifeArea::Career => "Career, public image, reputation, authority",
            LifeArea::Community => "Friends, groups, hopes, social causes",
            LifeArea::Subconscious => "Subconscious, hidden matters, isolation, endings",
        }
    }

    /// Get all life areas
    pub fn all() -> &'static [LifeArea] {
        &[
            LifeArea::Identity,
            LifeArea::Finances,
            LifeArea::Communication,
            LifeArea::Home,
            LifeArea::Romance,
            LifeArea::Work,
            LifeArea::Partnerships,
            LifeArea::Transformation,
            LifeArea::Spirituality,
            LifeArea::Career,
            LifeArea::Community,
            LifeArea::Subconscious,
        ]
    }
}

impl fmt::Display for LifeArea {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            LifeArea::Identity => "Identity",
            LifeArea::Finances => "Finances",
            LifeArea::Communication => "Communication",
            LifeArea::Home => "Home",
            LifeArea::Romance => "Romance",
            LifeArea::Work => "Work",
            LifeArea::Partnerships => "Partnerships",
            LifeArea::Transformation => "Transformation",
            LifeArea::Spirituality => "Spirituality",
            LifeArea::Career => "Career",
            LifeArea::Community => "Community",
            LifeArea::Subconscious => "Subconscious",
        };
        write!(f, "{}", name)
    }
}
