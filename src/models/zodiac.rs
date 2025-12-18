use serde::{Deserialize, Serialize};
use std::fmt;

/// Zodiac signs in order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ZodiacSign {
    Aries,
    Taurus,
    Gemini,
    Cancer,
    Leo,
    Virgo,
    Libra,
    Scorpio,
    Sagittarius,
    Capricorn,
    Aquarius,
    Pisces,
}

impl ZodiacSign {
    /// Get sign from ecliptic longitude (0-360 degrees)
    pub fn from_longitude(longitude: f64) -> Self {
        let normalized = longitude.rem_euclid(360.0);
        let sign_index = (normalized / 30.0).floor() as usize;
        Self::from_index(sign_index)
    }

    /// Get sign from index (0 = Aries, 11 = Pisces)
    pub fn from_index(index: usize) -> Self {
        match index % 12 {
            0 => ZodiacSign::Aries,
            1 => ZodiacSign::Taurus,
            2 => ZodiacSign::Gemini,
            3 => ZodiacSign::Cancer,
            4 => ZodiacSign::Leo,
            5 => ZodiacSign::Virgo,
            6 => ZodiacSign::Libra,
            7 => ZodiacSign::Scorpio,
            8 => ZodiacSign::Sagittarius,
            9 => ZodiacSign::Capricorn,
            10 => ZodiacSign::Aquarius,
            11 => ZodiacSign::Pisces,
            _ => unreachable!(),
        }
    }

    /// Get the starting degree of this sign (0 = Aries start)
    pub fn start_degree(&self) -> f64 {
        self.index() as f64 * 30.0
    }

    /// Get sign index (0 = Aries, 11 = Pisces)
    pub fn index(&self) -> usize {
        match self {
            ZodiacSign::Aries => 0,
            ZodiacSign::Taurus => 1,
            ZodiacSign::Gemini => 2,
            ZodiacSign::Cancer => 3,
            ZodiacSign::Leo => 4,
            ZodiacSign::Virgo => 5,
            ZodiacSign::Libra => 6,
            ZodiacSign::Scorpio => 7,
            ZodiacSign::Sagittarius => 8,
            ZodiacSign::Capricorn => 9,
            ZodiacSign::Aquarius => 10,
            ZodiacSign::Pisces => 11,
        }
    }

    /// Get next sign
    pub fn next(&self) -> Self {
        Self::from_index(self.index() + 1)
    }
}

impl fmt::Display for ZodiacSign {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            ZodiacSign::Aries => "Aries",
            ZodiacSign::Taurus => "Taurus",
            ZodiacSign::Gemini => "Gemini",
            ZodiacSign::Cancer => "Cancer",
            ZodiacSign::Leo => "Leo",
            ZodiacSign::Virgo => "Virgo",
            ZodiacSign::Libra => "Libra",
            ZodiacSign::Scorpio => "Scorpio",
            ZodiacSign::Sagittarius => "Sagittarius",
            ZodiacSign::Capricorn => "Capricorn",
            ZodiacSign::Aquarius => "Aquarius",
            ZodiacSign::Pisces => "Pisces",
        };
        write!(f, "{}", name)
    }
}

/// Celestial bodies used in astrological calculations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Planet {
    Sun,
    Moon,
    Mercury,
    Venus,
    Mars,
    Jupiter,
    Saturn,
    Uranus,
    Neptune,
    Pluto,
    /// North Node (True Lunar Node) - the ascending lunar node
    NorthNode,
}

impl Planet {
    /// Get all planets for iteration
    pub fn all() -> &'static [Planet] {
        &[
            Planet::Sun,
            Planet::Moon,
            Planet::Mercury,
            Planet::Venus,
            Planet::Mars,
            Planet::Jupiter,
            Planet::Saturn,
            Planet::Uranus,
            Planet::Neptune,
            Planet::Pluto,
            Planet::NorthNode,
        ]
    }

    /// Get Swiss Ephemeris body ID
    pub fn swe_id(&self) -> i32 {
        match self {
            Planet::Sun => 0,       // SE_SUN
            Planet::Moon => 1,      // SE_MOON
            Planet::Mercury => 2,   // SE_MERCURY
            Planet::Venus => 3,     // SE_VENUS
            Planet::Mars => 4,      // SE_MARS
            Planet::Jupiter => 5,   // SE_JUPITER
            Planet::Saturn => 6,    // SE_SATURN
            Planet::Uranus => 7,    // SE_URANUS
            Planet::Neptune => 8,   // SE_NEPTUNE
            Planet::Pluto => 9,     // SE_PLUTO
            Planet::NorthNode => 11, // SE_TRUE_NODE (True Lunar Node)
        }
    }

    /// Whether this body can be retrograde
    /// Sun and Moon cannot retrograde; North Node is always retrograde (apparent motion)
    pub fn can_retrograde(&self) -> bool {
        !matches!(self, Planet::Sun | Planet::Moon)
    }

    /// Whether this is a lunar node (for special handling)
    pub fn is_lunar_node(&self) -> bool {
        matches!(self, Planet::NorthNode)
    }
}

impl fmt::Display for Planet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Planet::Sun => "Sun",
            Planet::Moon => "Moon",
            Planet::Mercury => "Mercury",
            Planet::Venus => "Venus",
            Planet::Mars => "Mars",
            Planet::Jupiter => "Jupiter",
            Planet::Saturn => "Saturn",
            Planet::Uranus => "Uranus",
            Planet::Neptune => "Neptune",
            Planet::Pluto => "Pluto",
            Planet::NorthNode => "North Node",
        };
        write!(f, "{}", name)
    }
}

/// A position in the zodiac with sign and degree
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ZodiacPosition {
    pub sign: ZodiacSign,
    /// Degree within the sign (0-29.999...)
    pub degree: f64,
    /// Full ecliptic longitude (0-360)
    pub longitude: f64,
}

impl ZodiacPosition {
    pub fn from_longitude(longitude: f64) -> Self {
        let normalized = longitude.rem_euclid(360.0);
        let sign = ZodiacSign::from_longitude(normalized);
        let degree = normalized - sign.start_degree();
        Self {
            sign,
            degree,
            longitude: normalized,
        }
    }

    /// Format as "X° Sign" (e.g., "28° Scorpio")
    pub fn format_degree_sign(&self) -> String {
        format!("{}° {}", self.degree.round() as i32, self.sign)
    }
}
