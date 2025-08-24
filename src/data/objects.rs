use anyhow::Result;
use csv::{Reader, ReaderBuilder};
use flate2::read::GzDecoder;
use serde::Deserialize;
use std::collections::HashMap;

use crate::types::{
    hours_to_degrees, parse_dms, parse_hms, parse_or, sexagesimal_dms_to_degrees,
    sexagesimal_hms_to_hours, CelestialObject, EQPoint, Size,
};

// Embed the NGC catalog
pub const NGC_CSV_GZ: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/NGC.csv.gz"));

#[derive(Debug, Deserialize)]
struct NgcRow {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Type")]
    obj_type: String,
    #[serde(rename = "RA")]
    ra: String, // "HH:MM:SS.S"
    #[serde(rename = "Dec")]
    dec: String, // "+DD:MM:SS" or "-DD:MM:SS"
    #[serde(rename = "MajAx")]
    maj_ax: String,
    #[serde(rename = "MinAx")]
    min_ax: String,
    #[serde(rename = "PosAng")]
    pos_ang: String,
    #[serde(rename = "B-Mag")]
    bmag: String,
    #[serde(rename = "V-Mag")]
    vmag: String,
    #[serde(rename = "J-Mag")]
    jmag: String,
    #[serde(rename = "H-Mag")]
    hmag: String,
    #[serde(rename = "K-Mag")]
    kmag: String,
    #[serde(rename = "M", default)]
    m: Option<String>,
}

fn ngc_type_map() -> HashMap<&'static str, usize> {
    use std::iter::FromIterator;
    HashMap::from_iter([
        ("*", 0),
        ("**", 1),
        ("***", 2),
        ("*Ass", 9),
        ("OCl", 4),
        ("GCl", 5),
        ("Cl+N", 4),
        ("G", 3),
        ("GPair", 3),
        ("GTrpl", 3),
        ("GGroup", 3),
        ("PN", 6),
        ("HII", 7),
        ("DrkN", 7),
        ("EmN", 7),
        ("Neb", 7),
        ("RfN", 7),
        ("SNR", 6),
        ("Nova", 9),
        ("NonEx", 9),
        ("Dup", 9),
        ("Other", 9),
    ])
}
static OBJECT_TYPES: [&str; 10] = [
    "star",
    "double-star",
    "triple-star",
    "galaxy",
    "open-cluster",
    "globular-cluster",
    "planetary-nebula",
    "bright-nebula",
    "milky-way",
    "not-used",
];

pub fn load_objects(path: Option<&str>) -> Result<Vec<CelestialObject>> {
    if let Some(p) = path {
        let rdr = ReaderBuilder::new().delimiter(b';').from_path(p)?;
        parse_objects_from_reader(rdr)
    } else {
        let gz = GzDecoder::new(NGC_CSV_GZ);
        let rdr = ReaderBuilder::new().delimiter(b';').from_reader(gz);
        parse_objects_from_reader(rdr)
    }
}

/// Extract the first run of ASCII digits from a string, if any.
fn first_number(s: &str) -> Option<String> {
    let mut out = String::new();
    let mut seen = false;
    for ch in s.chars() {
        if ch.is_ascii_digit() {
            out.push(ch);
            seen = true;
        } else if seen {
            break;
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

/// Parse leading "NGC####" / "IC####" from Name.
/// Returns (catalog, number) when recognized; otherwise None.
fn parse_catalog_number_from_name(name: &str) -> Option<(String, String)> {
    let s = name.trim();
    if s.len() < 3 {
        return None;
    }

    let starts_with = |prefix: &str| {
        s.get(0..prefix.len())
            .map(|x| x.eq_ignore_ascii_case(prefix))
            .unwrap_or(false)
    };
    if starts_with("NGC") {
        if let Some(n) = first_number(s[3..].trim_start()) {
            return Some(("NGC".to_string(), n));
        }
    } else if starts_with("IC") {
        if let Some(n) = first_number(s[2..].trim_start()) {
            return Some(("IC".to_string(), n));
        }
    }
    None
}

/// Choose: M column first if it has a number,
/// otherwise parse name for NGC/IC,
/// finally fallback to just the name
fn choose_catalog_and_identifier(m: &Option<String>, name: &str) -> (String, String) {
    if let Some(id) = m.as_deref().map(str::trim).filter(|t| !t.is_empty()) {
        // Use M verbatim (trimmed), e.g. "042" stays "042"
        return ("M".to_string(), id.trim_start_matches('0').to_string());
    }
    if let Some((cat, id)) = parse_catalog_number_from_name(name) {
        return (cat, id);
    }
    ("Unknown".to_string(), name.trim().to_string())
}

fn parse_objects_from_reader<R: std::io::Read>(mut rdr: Reader<R>) -> Result<Vec<CelestialObject>> {
    let mut out = Vec::new();
    let type_map = ngc_type_map();

    for rec in rdr.deserialize() {
        let row: NgcRow = rec?;
        if row.ra.trim().is_empty() || row.dec.trim().is_empty() {
            continue;
        }

        let idx = *type_map.get(row.obj_type.as_str()).unwrap_or(&9);
        let kind = OBJECT_TYPES[idx];
        if kind.contains("star") {
            continue;
        }

        let magnitude = [
            parse_or::<f64>(&row.bmag, 20.0),
            parse_or::<f64>(&row.vmag, 20.0),
            parse_or::<f64>(&row.jmag, 20.0),
            parse_or::<f64>(&row.hmag, 20.0),
            parse_or::<f64>(&row.kmag, 20.0),
        ]
        .into_iter()
        .fold(f64::INFINITY, |acc, v| acc.min(v));

        let angle = parse_or::<f64>(&row.pos_ang, 0.0);
        let size = Size {
            major: parse_or::<f64>(&row.maj_ax, 0.0),
            minor: parse_or::<f64>(&row.min_ax, 0.0),
        };

        let (h, m, s) = match parse_hms(&row.ra) {
            Some(t) => t,
            None => continue,
        };
        let (d, dm, ds) = match parse_dms(&row.dec) {
            Some(t) => t,
            None => continue,
        };
        let ra_deg = hours_to_degrees(sexagesimal_hms_to_hours(h, m, s));
        let dec_deg = sexagesimal_dms_to_degrees(d, dm, ds);

        // Chose which catalog and label to use for this object.
        // This simply prefers the Messier identifier if it exists.
        let (catalog, identifier) = choose_catalog_and_identifier(&row.m, &row.name);

        out.push(CelestialObject {
            kind: kind.to_string(),
            catalog: catalog,
            identifier: identifier,
            coords: EQPoint { ra_deg, dec_deg },
            magnitude: magnitude,
            size: size,
            angle: angle,
            name: String::new(),
        });
    }

    // Sort by magnitude, reverse=True, for drawing later
    out.sort_by(|a, b| {
        a.magnitude
            .partial_cmp(&b.magnitude)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out.reverse();
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messier_from_m_string() {
        assert_eq!(
            choose_catalog_and_identifier(&Some("42".into()), "NGC 1976"),
            ("M".into(), "42".into())
        );
        // preserves leading zeros (after trim)
        assert_eq!(
            choose_catalog_and_identifier(&Some("  042  ".into()), "IC 434"),
            ("M".into(), "42".into())
        );
    }

    #[test]
    fn parse_ngc_ic_from_name_when_no_m() {
        assert_eq!(
            choose_catalog_and_identifier(&None, "NGC 1976"),
            ("NGC".into(), "1976".into())
        );
        assert_eq!(
            choose_catalog_and_identifier(&None, "IC434"),
            ("IC".into(), "434".into())
        );
    }

    #[test]
    fn fallback_to_name_when_unrecognized() {
        assert_eq!(
            choose_catalog_and_identifier(&None, "SH2123"),
            ("Unknown".into(), "SH2123".into())
        );
    }
}
