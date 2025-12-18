//! House system calculations using Swiss Ephemeris

use crate::models::ZodiacPosition;

// House system codes for Swiss Ephemeris
#[allow(dead_code)]
pub const HOUSE_PLACIDUS: i8 = b'P' as i8;
#[allow(dead_code)]
pub const HOUSE_KOCH: i8 = b'K' as i8;
#[allow(dead_code)]
pub const HOUSE_EQUAL: i8 = b'E' as i8;
#[allow(dead_code)]
pub const HOUSE_WHOLE_SIGN: i8 = b'W' as i8;

/// Result of house calculation
#[derive(Debug, Clone)]
pub struct HousePositions {
    /// Ascendant (1st house cusp)
    pub ascendant: f64,
    /// Midheaven (10th house cusp)
    pub midheaven: f64,
    /// House cusps (12 houses, index 0 = 1st house)
    pub cusps: [f64; 12],
    /// ARMC (Sidereal time at location)
    pub armc: f64,
    /// Vertex
    pub vertex: f64,
}

impl HousePositions {
    pub fn ascendant_position(&self) -> ZodiacPosition {
        ZodiacPosition::from_longitude(self.ascendant)
    }

    pub fn midheaven_position(&self) -> ZodiacPosition {
        ZodiacPosition::from_longitude(self.midheaven)
    }
}

/// Calculate house positions for a given time and location
pub fn calc_houses(
    julian_day: f64,
    latitude: f64,
    longitude: f64,
    house_system: i8,
) -> Result<HousePositions, String> {
    // Swiss Ephemeris uses a 13-element array for cusps (index 1-12)
    // and a 10-element array for special points
    let mut cusps: [f64; 13] = [0.0; 13];
    let mut ascmc: [f64; 10] = [0.0; 10];

    let ret = unsafe {
        libswisseph_sys::swe_houses(
            julian_day,
            latitude,
            longitude,
            house_system as i32,
            cusps.as_mut_ptr(),
            ascmc.as_mut_ptr(),
        )
    };

    if ret < 0 {
        return Err("Failed to calculate houses".to_string());
    }

    // Convert from 1-indexed to 0-indexed for our cusps array
    let mut house_cusps: [f64; 12] = [0.0; 12];
    for i in 0..12 {
        house_cusps[i] = cusps[i + 1];
    }

    Ok(HousePositions {
        ascendant: ascmc[0],
        midheaven: ascmc[1],
        cusps: house_cusps,
        armc: ascmc[2],
        vertex: ascmc[3],
    })
}

/// Calculate Ascendant and Midheaven only (faster than full house calculation)
pub fn calc_asc_mc(
    julian_day: f64,
    latitude: f64,
    longitude: f64,
) -> Result<(ZodiacPosition, ZodiacPosition), String> {
    let houses = calc_houses(julian_day, latitude, longitude, HOUSE_PLACIDUS)?;
    Ok((houses.ascendant_position(), houses.midheaven_position()))
}

/// Determine which house (1-12) a planet is in based on house cusps
/// Uses the Placidus-style house determination where a planet is in a house
/// if its longitude is between that house cusp and the next house cusp
pub fn planet_in_house(planet_longitude: f64, house_cusps: &[f64; 12]) -> u8 {
    let lon = planet_longitude.rem_euclid(360.0);

    for i in 0..12 {
        let cusp_start = house_cusps[i];
        let cusp_end = house_cusps[(i + 1) % 12];

        // Handle wrap-around at 360°/0°
        let in_house = if cusp_start <= cusp_end {
            // Normal case: cusp_start < planet < cusp_end
            lon >= cusp_start && lon < cusp_end
        } else {
            // Wrap-around case: e.g., cusp_start = 350°, cusp_end = 20°
            lon >= cusp_start || lon < cusp_end
        };

        if in_house {
            return (i + 1) as u8;
        }
    }

    // Fallback (shouldn't happen with valid data)
    1
}

/// Get house system name from code
pub fn house_system_name(code: i8) -> &'static str {
    match code as u8 as char {
        'P' => "Placidus",
        'K' => "Koch",
        'E' => "Equal",
        'W' => "Whole Sign",
        'R' => "Regiomontanus",
        'C' => "Campanus",
        'B' => "Alcabitius",
        'M' => "Morinus",
        'O' => "Porphyry",
        _ => "Unknown",
    }
}
