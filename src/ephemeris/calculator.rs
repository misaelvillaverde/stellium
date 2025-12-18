//! Swiss Ephemeris wrapper for planetary calculations
//!
//! This module provides safe Rust wrappers around the libswisseph-sys FFI bindings.

use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike, Utc};
use chrono_tz::Tz;
use std::sync::Once;

use crate::models::{Planet, ZodiacPosition, ZodiacSign};

// Swiss Ephemeris constants
const SE_GREG_CAL: i32 = 1;
const SEFLG_SPEED: i32 = 256; // Include speed in calculations
const SEFLG_SWIEPH: i32 = 2; // Use Swiss Ephemeris

static INIT: Once = Once::new();

/// Initialize Swiss Ephemeris (call once at startup)
pub fn init_ephemeris() {
    INIT.call_once(|| {
        unsafe {
            // Initialize without ephemeris files (uses Moshier analytical ephemeris)
            // This provides 0.1 arc seconds precision for planets, 3 arc seconds for Moon
            libswisseph_sys::swe_set_ephe_path(std::ptr::null_mut());
        }
    });
}

/// Result of a planetary calculation
#[derive(Debug, Clone)]
pub struct PlanetaryPosition {
    /// Ecliptic longitude (0-360 degrees)
    pub longitude: f64,
    /// Ecliptic latitude
    pub latitude: f64,
    /// Distance (AU for planets, Earth radii for Moon)
    pub distance: f64,
    /// Speed in longitude (degrees per day, negative = retrograde)
    pub speed_longitude: f64,
    /// Speed in latitude
    pub speed_latitude: f64,
    /// Speed in distance
    pub speed_distance: f64,
    /// Whether the planet is retrograde
    pub is_retrograde: bool,
}

impl PlanetaryPosition {
    pub fn to_zodiac_position(&self) -> ZodiacPosition {
        ZodiacPosition::from_longitude(self.longitude)
    }
}

/// Convert a date/time to Julian Day (UT)
pub fn datetime_to_julian_day(datetime: NaiveDateTime) -> f64 {
    let year = datetime.date().year();
    let month = datetime.date().month() as i32;
    let day = datetime.date().day() as i32;
    let hour = datetime.time().hour() as f64
        + datetime.time().minute() as f64 / 60.0
        + datetime.time().second() as f64 / 3600.0;

    unsafe { libswisseph_sys::swe_julday(year, month, day, hour, SE_GREG_CAL) }
}

/// Convert a date to Julian Day (at midnight UT)
pub fn date_to_julian_day(date: NaiveDate) -> f64 {
    let datetime = date.and_hms_opt(0, 0, 0).unwrap();
    datetime_to_julian_day(datetime)
}

/// Convert a local date/time with timezone to Julian Day (UT)
pub fn local_datetime_to_julian_day(
    date: NaiveDate,
    time: NaiveTime,
    timezone: &str,
) -> Result<f64, String> {
    let tz: Tz = timezone
        .parse()
        .map_err(|_| format!("Invalid timezone: {}", timezone))?;

    let local_datetime = NaiveDateTime::new(date, time);

    // Convert local time to UTC
    let local_dt = tz
        .from_local_datetime(&local_datetime)
        .single()
        .ok_or_else(|| "Ambiguous or invalid local time".to_string())?;

    let utc_datetime = local_dt.with_timezone(&Utc).naive_utc();

    Ok(datetime_to_julian_day(utc_datetime))
}

/// Calculate position of a planet at a given Julian Day
pub fn calc_planet_position(planet: Planet, julian_day: f64) -> Result<PlanetaryPosition, String> {
    init_ephemeris();

    let mut xx: [f64; 6] = [0.0; 6];
    let mut serr: [i8; 256] = [0; 256];

    let iflg = SEFLG_SPEED | SEFLG_SWIEPH;

    let ret = unsafe {
        libswisseph_sys::swe_calc_ut(
            julian_day,
            planet.swe_id(),
            iflg,
            xx.as_mut_ptr(),
            serr.as_mut_ptr(),
        )
    };

    if ret < 0 {
        let error_msg = unsafe {
            let c_str = std::ffi::CStr::from_ptr(serr.as_ptr());
            c_str.to_string_lossy().to_string()
        };
        return Err(format!("Swiss Ephemeris error: {}", error_msg));
    }

    let speed_longitude = xx[3];
    let is_retrograde = planet.can_retrograde() && speed_longitude < 0.0;

    Ok(PlanetaryPosition {
        longitude: xx[0],
        latitude: xx[1],
        distance: xx[2],
        speed_longitude,
        speed_latitude: xx[4],
        speed_distance: xx[5],
        is_retrograde,
    })
}

/// Calculate positions for all planets at a given Julian Day
pub fn calc_all_planets(julian_day: f64) -> Result<Vec<(Planet, PlanetaryPosition)>, String> {
    let mut positions = Vec::new();

    for planet in Planet::all() {
        let position = calc_planet_position(*planet, julian_day)?;
        positions.push((*planet, position));
    }

    Ok(positions)
}

/// Find the date when a planet enters a new sign (searching forward)
pub fn find_next_sign_ingress(
    planet: Planet,
    start_julian_day: f64,
    max_days: i32,
) -> Result<Option<(f64, ZodiacSign)>, String> {
    let start_pos = calc_planet_position(planet, start_julian_day)?;
    let start_sign = ZodiacSign::from_longitude(start_pos.longitude);

    // Step size depends on planet speed (faster planets need smaller steps)
    let step = match planet {
        Planet::Moon => 0.5, // Moon moves ~13° per day
        Planet::Sun | Planet::Mercury | Planet::Venus => 1.0,
        Planet::Mars => 2.0,
        _ => 5.0, // Outer planets move slowly
    };

    let mut jd = start_julian_day;
    let end_jd = start_julian_day + max_days as f64;

    while jd < end_jd {
        let pos = calc_planet_position(planet, jd)?;
        let current_sign = ZodiacSign::from_longitude(pos.longitude);

        if current_sign != start_sign {
            // Found sign change, refine with binary search
            let mut low = jd - step;
            let mut high = jd;

            while high - low > 0.001 {
                // ~1.4 minutes precision
                let mid = (low + high) / 2.0;
                let mid_pos = calc_planet_position(planet, mid)?;
                let mid_sign = ZodiacSign::from_longitude(mid_pos.longitude);

                if mid_sign == start_sign {
                    low = mid;
                } else {
                    high = mid;
                }
            }

            return Ok(Some((high, current_sign)));
        }

        jd += step;
    }

    Ok(None)
}

/// Find the date when a planet turns retrograde or direct
pub fn find_next_station(
    planet: Planet,
    start_julian_day: f64,
    max_days: i32,
) -> Result<Option<(f64, bool)>, String> {
    if !planet.can_retrograde() {
        return Ok(None);
    }

    let start_pos = calc_planet_position(planet, start_julian_day)?;
    let start_retrograde = start_pos.is_retrograde;

    let step = 1.0; // Check daily

    let mut jd = start_julian_day;
    let end_jd = start_julian_day + max_days as f64;

    while jd < end_jd {
        let pos = calc_planet_position(planet, jd)?;

        if pos.is_retrograde != start_retrograde {
            // Found station, refine
            let mut low = jd - step;
            let mut high = jd;

            while high - low > 0.01 {
                // ~15 minutes precision
                let mid = (low + high) / 2.0;
                let mid_pos = calc_planet_position(planet, mid)?;

                if mid_pos.is_retrograde == start_retrograde {
                    low = mid;
                } else {
                    high = mid;
                }
            }

            // Returns true if turning retrograde, false if turning direct
            return Ok(Some((high, !start_retrograde)));
        }

        jd += step;
    }

    Ok(None)
}

/// Calculate the Sun-Moon angle (for lunar phases)
pub fn calc_sun_moon_angle(julian_day: f64) -> Result<f64, String> {
    let sun = calc_planet_position(Planet::Sun, julian_day)?;
    let moon = calc_planet_position(Planet::Moon, julian_day)?;

    // Moon longitude - Sun longitude, normalized to 0-360
    let angle = (moon.longitude - sun.longitude).rem_euclid(360.0);
    Ok(angle)
}

/// Find the next new moon (Sun-Moon conjunction)
pub fn find_next_new_moon(start_julian_day: f64, max_days: i32) -> Result<Option<f64>, String> {
    find_next_lunar_phase(start_julian_day, 0.0, max_days)
}

/// Find the next full moon (Sun-Moon opposition)
pub fn find_next_full_moon(start_julian_day: f64, max_days: i32) -> Result<Option<f64>, String> {
    find_next_lunar_phase(start_julian_day, 180.0, max_days)
}

/// Find the next occurrence of a specific lunar phase angle
fn find_next_lunar_phase(
    start_julian_day: f64,
    target_angle: f64,
    max_days: i32,
) -> Result<Option<f64>, String> {
    let step = 0.5; // Check every 12 hours
    let mut jd = start_julian_day;
    let end_jd = start_julian_day + max_days as f64;

    let mut prev_angle = calc_sun_moon_angle(jd)?;

    while jd < end_jd {
        jd += step;
        let current_angle = calc_sun_moon_angle(jd)?;

        // Check if we crossed the target angle
        let crossed = if target_angle < 10.0 || target_angle > 350.0 {
            // Handle wrap-around for new moon (0°)
            (prev_angle > 350.0 && current_angle < 10.0)
                || (prev_angle < target_angle + 10.0
                    && current_angle >= target_angle
                    && current_angle < target_angle + 20.0)
        } else {
            prev_angle < target_angle && current_angle >= target_angle
        };

        if crossed {
            // Refine with binary search
            let mut low = jd - step;
            let mut high = jd;

            while high - low > 0.001 {
                let mid = (low + high) / 2.0;
                let mid_angle = calc_sun_moon_angle(mid)?;

                let before_target = if target_angle < 10.0 {
                    mid_angle > 180.0 // Before new moon means angle > 180
                } else {
                    mid_angle < target_angle
                };

                if before_target {
                    low = mid;
                } else {
                    high = mid;
                }
            }

            return Ok(Some(high));
        }

        prev_angle = current_angle;
    }

    Ok(None)
}

/// Convert Julian Day back to NaiveDate
pub fn julian_day_to_date(julian_day: f64) -> NaiveDate {
    let mut year: i32 = 0;
    let mut month: i32 = 0;
    let mut day: i32 = 0;
    let mut hour: f64 = 0.0;

    unsafe {
        libswisseph_sys::swe_revjul(
            julian_day,
            SE_GREG_CAL,
            &mut year,
            &mut month,
            &mut day,
            &mut hour,
        );
    }

    NaiveDate::from_ymd_opt(year, month as u32, day as u32)
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(2000, 1, 1).unwrap())
}

/// Convert Julian Day to NaiveDateTime
pub fn julian_day_to_datetime(julian_day: f64) -> NaiveDateTime {
    let mut year: i32 = 0;
    let mut month: i32 = 0;
    let mut day: i32 = 0;
    let mut hour: f64 = 0.0;

    unsafe {
        libswisseph_sys::swe_revjul(
            julian_day,
            SE_GREG_CAL,
            &mut year,
            &mut month,
            &mut day,
            &mut hour,
        );
    }

    let hours = hour.floor() as u32;
    let minutes = ((hour - hours as f64) * 60.0).floor() as u32;
    let seconds = (((hour - hours as f64) * 60.0 - minutes as f64) * 60.0).floor() as u32;

    let date = NaiveDate::from_ymd_opt(year, month as u32, day as u32)
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());
    let time = NaiveTime::from_hms_opt(hours, minutes, seconds)
        .unwrap_or_else(|| NaiveTime::from_hms_opt(0, 0, 0).unwrap());

    NaiveDateTime::new(date, time)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_julian_day_conversion() {
        let date = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
        let jd = date_to_julian_day(date);
        // J2000.0 epoch is Julian Day 2451545.0
        assert!((jd - 2451544.5).abs() < 0.01);
    }

    #[test]
    fn test_planet_calculation() {
        init_ephemeris();
        let jd = date_to_julian_day(NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());
        let sun = calc_planet_position(Planet::Sun, jd).unwrap();

        // Sun should be around 280° (Capricorn) on Jan 1, 2000
        assert!(sun.longitude > 270.0 && sun.longitude < 290.0);
    }

    #[test]
    fn test_zodiac_sign() {
        let sign = ZodiacSign::from_longitude(280.0);
        assert_eq!(sign, ZodiacSign::Capricorn);

        let sign = ZodiacSign::from_longitude(45.0);
        assert_eq!(sign, ZodiacSign::Taurus);
    }
}
