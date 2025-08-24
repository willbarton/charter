#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EQPoint {
    pub ra_deg: f64,
    pub dec_deg: f64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Size {
    pub major: f64,
    pub minor: f64,
}

impl Size {
    pub fn zero() -> Self {
        Self {
            major: 0.0,
            minor: 0.0,
        }
    }
}

pub struct Constellation {
    pub name: String,
    pub lines: Vec<Vec<EQPoint>>,
}

#[derive(Debug, Clone)]
pub struct CelestialObject {
    pub kind: String,
    pub catalog: String,
    pub identifier: String,
    pub coords: EQPoint,
    pub magnitude: f64,
    pub size: Size,
    pub angle: f64,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Projection {
    Gnomonic,
    Stereographic,
    Spherical,
    AltAz,
}

impl Projection {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "gnomonic" => Some(Self::Gnomonic),
            "stereographic" => Some(Self::Stereographic),
            "spherical" => Some(Self::Spherical),
            "altaz" => Some(Self::AltAz),
            _ => None,
        }
    }
}

// Small helpers used by multiple modules
pub fn parse_or<T: std::str::FromStr>(s: &str, default: T) -> T {
    s.parse::<T>().unwrap_or(default)
}

pub fn hours_to_degrees(hours: f64) -> f64 {
    hours * 15.0
}

pub fn sexagesimal_hms_to_hours(h: f64, m: f64, s: f64) -> f64 {
    h + (m * 60.0 + s) / 3600.0
}

pub fn sexagesimal_dms_to_degrees(d: f64, m: f64, s: f64) -> f64 {
    let sign = if d.is_sign_negative() { -1.0 } else { 1.0 };
    let ad = d.abs();
    sign * (ad + (m * 60.0 + s) / 3600.0)
}

pub fn parse_hms(s: &str) -> Option<(f64, f64, f64)> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 3 {
        return None;
    }
    Some((
        parse_or(parts[0], 0.0),
        parse_or(parts[1], 0.0),
        parse_or(parts[2], 0.0),
    ))
}

pub fn parse_dms(s: &str) -> Option<(f64, f64, f64)> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 3 {
        return None;
    }
    Some((
        parse_or(parts[0], 0.0),
        parse_or(parts[1], 0.0),
        parse_or(parts[2], 0.0),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::approx;

    #[test]
    fn projection_from_str_recognizes_known_values() {
        assert_eq!(Projection::from_str("gnomonic"), Some(Projection::Gnomonic));
        assert_eq!(
            Projection::from_str("stereographic"),
            Some(Projection::Stereographic)
        );
        assert_eq!(
            Projection::from_str("spherical"),
            Some(Projection::Spherical)
        );
        assert_eq!(Projection::from_str("altaz"), Some(Projection::AltAz));
    }

    #[test]
    fn projection_from_str_is_case_sensitive_and_handles_unknown() {
        assert_eq!(Projection::from_str("Gnomonic"), None);
        assert_eq!(Projection::from_str("unknown"), None);
        assert_eq!(Projection::from_str(""), None);
    }

    #[test]
    fn parse_or_parses_ints_and_defaults_on_error() {
        let v: i32 = parse_or("42", 0);
        assert_eq!(v, 42);

        let v_bad: i32 = parse_or("", 7);
        assert_eq!(v_bad, 7);

        let v_bad2: i32 = parse_or("not-an-int", -5);
        assert_eq!(v_bad2, -5);
    }

    #[test]
    fn parse_or_parses_floats_and_defaults_on_error() {
        let v: f64 = parse_or("3.14159", 0.0);
        assert!(approx(v, 3.14159, 1e-12));

        let v_bad: f64 = parse_or("oops", 1.23);
        assert!(approx(v_bad, 1.23, 1e-12));
    }

    #[test]
    fn hours_to_degrees_basic_and_negative() {
        assert!(approx(hours_to_degrees(0.0), 0.0, 1e-12));
        assert!(approx(hours_to_degrees(1.0), 15.0, 1e-12));
        assert!(approx(hours_to_degrees(6.5), 97.5, 1e-12));
        assert!(approx(hours_to_degrees(-2.0), -30.0, 1e-12));
    }

    #[test]
    fn hms_to_hours_converts_correctly() {
        // 1h 30m 0s = 1.5h
        assert!(approx(sexagesimal_hms_to_hours(1.0, 30.0, 0.0), 1.5, 1e-12));
        // 0h 0m 30s = 0.008333...h
        assert!(approx(
            sexagesimal_hms_to_hours(0.0, 0.0, 30.0),
            30.0 / 3600.0,
            1e-15
        ));
        // 23h 59m 59.9s ≈ 23.9999722h
        let v = sexagesimal_hms_to_hours(23.0, 59.0, 59.9);
        assert!(approx(v, 23.0 + (59.0 * 60.0 + 59.9) / 3600.0, 1e-12));
    }

    #[test]
    fn dms_to_degrees_positive_and_negative() {
        // +10° 30' 0" = +10.5°
        assert!(approx(
            sexagesimal_dms_to_degrees(10.0, 30.0, 0.0),
            10.5,
            1e-12
        ));
        // -10° 30' 0" = -10.5°
        assert!(approx(
            sexagesimal_dms_to_degrees(-10.0, 30.0, 0.0),
            -10.5,
            1e-12
        ));
        // +0° 30' 0" = +0.5°
        assert!(approx(
            sexagesimal_dms_to_degrees(0.0, 30.0, 0.0),
            0.5,
            1e-12
        ));
    }

    #[test]
    fn dms_handles_negative_zero_degrees() {
        // -0° 30' 0" should be -0.5°
        let result = sexagesimal_dms_to_degrees(-0.0, 30.0, 0.0);
        assert!(approx(result, -0.5, 1e-12));
    }

    #[test]
    fn parse_hms_ok_and_wrong_lengths() {
        // OK
        let (h, m, s) = parse_hms("12:34:56.7").expect("should parse");
        assert!(approx(h, 12.0, 1e-12));
        assert!(approx(m, 34.0, 1e-12));
        assert!(approx(s, 56.7, 1e-12));

        // wrong part counts
        assert!(parse_hms("12:34").is_none());
        assert!(parse_hms("12:34:56:78").is_none());
        assert!(parse_hms("").is_none());
    }

    #[test]
    fn parse_dms_ok_and_wrong_lengths() {
        // OK
        let (d, m, s) = parse_dms("-10:30:00").expect("should parse");
        assert!(approx(d, -10.0, 1e-12));
        assert!(approx(m, 30.0, 1e-12));
        assert!(approx(s, 0.0, 1e-12));

        // wrong part counts
        assert!(parse_dms("10:30").is_none());
        assert!(parse_dms("10:30:00:00").is_none());
        assert!(parse_dms("xx").is_none());
    }
}
