use anyhow::Result;
use csv::{Reader, ReaderBuilder};
use flate2::read::GzDecoder;
use serde::Deserialize;

use crate::types::{hours_to_degrees, parse_or, CelestialObject, EQPoint, Size};

// Embed the gzipped star catalog
pub const HYG_CSV_GZ: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/hygdata.csv.gz"));

#[derive(Debug, Deserialize)]
struct HygRow {
    id: String,
    ra: String,
    dec: String,
    mag: String,
    proper: String,
}

fn parse_stars_from_reader<R: std::io::Read>(mut rdr: Reader<R>) -> Result<Vec<CelestialObject>> {
    let mut out = Vec::new();
    for rec in rdr.deserialize() {
        let row: HygRow = rec?;
        let ra_h: f64 = parse_or(&row.ra, 0.0);
        let dec_deg: f64 = parse_or(&row.dec, 0.0);
        let mag: f64 = parse_or(&row.mag, 99.0);
        out.push(CelestialObject {
            kind: "star".to_string(),
            catalog: "HYG".to_string(),
            identifier: row.id,
            coords: EQPoint {
                ra_deg: hours_to_degrees(ra_h),
                dec_deg,
            },
            magnitude: mag,
            size: Size::zero(),
            angle: 0.0,
            name: row.proper,
        });
    }
    Ok(out)
}

pub fn load_stars(path: Option<&str>) -> Result<Vec<CelestialObject>> {
    if let Some(p) = path {
        let rdr = ReaderBuilder::new().from_path(p)?;
        parse_stars_from_reader(rdr)
    } else {
        let gz = GzDecoder::new(HYG_CSV_GZ);
        let rdr = ReaderBuilder::new().from_reader(gz);
        parse_stars_from_reader(rdr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::approx;
    use csv::ReaderBuilder;

    fn parse_from_str(csv: &str) -> Vec<CelestialObject> {
        let rdr = ReaderBuilder::new().from_reader(csv.as_bytes());
        parse_stars_from_reader(rdr).expect("parse HYG CSV")
    }

    #[test]
    fn parses_rows_and_converts_ra_hours_to_degrees() {
        let csv = "\
id,ra,dec,mag,proper
32263,6.752481,-16.716116,-1.44,Sirius
27919,5.919529,7.407063,0.45,Betelgeuse
";
        let stars = parse_from_str(csv);
        assert_eq!(stars.len(), 2);

        // Row 1
        let s1 = &stars[0];
        assert_eq!(s1.kind, "star");
        assert_eq!(s1.catalog, "HYG");
        assert_eq!(s1.identifier, "32263");
        assert_eq!(s1.name, "Sirius");
        assert!(approx(s1.coords.ra_deg, 6.752481 * 15.0, 1e-10)); // hours → degrees
        assert!(approx(s1.coords.dec_deg, -16.716116, 1e-10));
        assert!(approx(s1.magnitude, -1.44, 1e-10));

        // Row 2 sanity check
        let s2 = &stars[1];
        assert!(approx(s2.coords.ra_deg, 5.919529 * 15.0, 1e-10));
        assert!(approx(s2.coords.dec_deg, 7.407063, 1e-10));
        assert!(approx(s2.magnitude, 0.45, 1e-10));
    }

    #[test]
    fn empty_magnitude_defaults_to_99() {
        let csv = "\
id,ra,dec,mag,proper
1,1.0,2.0,,
";
        let stars = parse_from_str(csv);
        assert_eq!(stars.len(), 1);
        let s = &stars[0];
        assert_eq!(s.name, ""); // empty proper carried through
        assert!(approx(s.coords.ra_deg, 15.0, 1e-12)); // 1h → 15°
        assert!(approx(s.coords.dec_deg, 2.0, 1e-12));
        assert!(approx(s.magnitude, 99.0, 1e-12)); // default
    }
}
