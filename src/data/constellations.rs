use anyhow::Result;
use csv::{Reader, ReaderBuilder, Trim};
use phf::phf_map;
use std::collections::HashMap;

use crate::types::{hours_to_degrees, Constellation, EQPoint};

// Embed the constellation data
pub const CONSTELLATIONS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/constellations.csv"
));

static CONSTELLATION_NAMES: phf::Map<&'static str, &'static str> = phf_map! {
    "AND" => "Andromeda",
    "ANT" => "Antlia",
    "APS" => "Apus",
    "AQR" => "Aquarius",
    "AQL" => "Aquila",
    "ARA" => "Ara",
    "LEO" => "Leo",
    "LMI" => "Leo Minor",
    "LEP" => "Lepus",
    "LIB" => "Libra",
    "LUP" => "Lupus",
    "LYN" => "Lynx",
    "LYR" => "Lyra",
    "MEN" => "Mensa",
    "MIC" => "Microscopium",
    "MON" => "Monoceros",
    "ARI" => "Aries",
    "AUR" => "Auriga",
    "BOO" => "Boötes",
    "CAE" => "Caelum",
    "CAM" => "Camelopardalis",
    "CNC" => "Cancer",
    "CVN" => "Canes Venatici",
    "CMA" => "Canis Major",
    "CMI" => "Canis Minor",
    "CAP" => "Capricornus",
    "CAR" => "Carina",
    "CAS" => "Cassiopeia",
    "CEN" => "Centaurus",
    "CEP" => "Cepheus",
    "CET" => "Cetus",
    "CHA" => "Chamaeleon",
    "CIR" => "Circinus",
    "COL" => "Columba",
    "COM" => "Coma Berenices",
    "CRA" => "Corona Australis",
    "CRB" => "Corona Borealis",
    "CRV" => "Corvus",
    "CRT" => "Crater",
    "CRU" => "Crux",
    "CYG" => "Cygnus",
    "DEL" => "Delphinus",
    "DOR" => "Dorado",
    "DRA" => "Draco",
    "EQU" => "Equuleus",
    "ERI" => "Eridanus",
    "FOR" => "Fornax",
    "GEM" => "Gemini",
    "GRU" => "Grus",
    "HER" => "Hercules",
    "HOR" => "Horologium",
    "HYA" => "Hydra",
    "HYI" => "Hydrus",
    "IND" => "Indus",
    "LAC" => "Lacerta",
    "MUS" => "Musca",
    "NOR" => "Norma",
    "OCT" => "Octans",
    "OPH" => "Ophiuchus",
    "ORI" => "Orion",
    "PAV" => "Pavo",
    "PEG" => "Pegasus",
    "PER" => "Perseus",
    "PHE" => "Phoenix",
    "PIC" => "Pictor",
    "PSC" => "Pisces",
    "PSA" => "Piscis Austrinus",
    "PUP" => "Puppis",
    "PYX" => "Pyxis",
    "RET" => "Reticulum",
    "SGE" => "Sagitta",
    "SGR" => "Sagittarius",
    "SCO" => "Scorpius",
    "SCL" => "Sculptor",
    "SCT" => "Scutum",
    // TODO: Nothing about this handles the fact that there are
    // two constellations with the abbreviation SER,
    // Serpens Caput and Serpens Cauda.
    "SER" => "Serpens",
    "SEX" => "Sextans",
    "TAU" => "Taurus",
    "TEL" => "Telescopium",
    "TRI" => "Triangulum",
    "TRA" => "Triangulum Australe",
    "TUC" => "Tucana",
    "UMA" => "Ursa Major",
    "UMI" => "Ursa Minor",
    "VEL" => "Vela",
    "VIR" => "Virgo",
    "VOL" => "Volans",
    "VUL" => "Vulpecula",
};

/// Load constellations
pub fn load_constellations(path: Option<&str>) -> Result<Vec<Constellation>> {
    if let Some(p) = path {
        let rdr = ReaderBuilder::new()
            .has_headers(false)
            .flexible(true) // variable-length rows
            .trim(Trim::All)
            .from_path(p)?;
        parse_constellations_from_reader(rdr)
    } else {
        let rdr = ReaderBuilder::new()
            .has_headers(false)
            .flexible(true) // variable-length rows
            .trim(Trim::All)
            .from_reader(CONSTELLATIONS_CSV.as_bytes());
        parse_constellations_from_reader(rdr)
    }
}

// The data for each constellation is in spread across multiple rows.
// The first column is the abbreviation, and the subsequent columns are pairs
// of RA and dec coordinates. There is a variable number of these pairs in
// each row.
fn parse_constellations_from_reader<R: std::io::Read>(
    mut rdr: Reader<R>,
) -> Result<Vec<Constellation>> {
    let mut by_abbr: HashMap<String, Constellation> = HashMap::new();

    for result in rdr.records() {
        let rec = result?;
        // Should be at least an abbreviation + at least one coordinate pair
        if rec.len() < 3 {
            continue;
        }

        let abbr = rec.get(0).unwrap().trim().to_string();
        let name = CONSTELLATION_NAMES
            .get(abbr.as_str())
            .copied()
            .unwrap_or("");

        let entry = by_abbr
            .entry(abbr.clone())
            .or_insert_with(|| Constellation {
                name: name.to_string(),
                lines: Vec::new(),
            });

        // Parse remaining fields as (RA_hours, Dec_deg) pairs.
        let mut line: Vec<EQPoint> = Vec::new();
        let mut i = 1usize;
        while i + 1 < rec.len() {
            let ra_s = rec.get(i).unwrap_or("").trim();
            let dec_s = rec.get(i + 1).unwrap_or("").trim();
            if !(ra_s.is_empty() || dec_s.is_empty()) {
                if let (Ok(ra_h), Ok(dec_deg)) = (ra_s.parse::<f64>(), dec_s.parse::<f64>()) {
                    line.push(EQPoint {
                        ra_deg: hours_to_degrees(ra_h),
                        dec_deg,
                    });
                }
            }
            i += 2;
        }

        if !line.is_empty() {
            entry.lines.push(line);
        }
    }

    Ok(by_abbr.into_values().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::approx;
    use csv::ReaderBuilder;

    // Create a CSV reader from a string and parse it for testing
    fn parse_from_str(s: &str) -> Vec<Constellation> {
        let rdr = ReaderBuilder::new()
            .has_headers(false)
            .flexible(true)
            .trim(Trim::All)
            .from_reader(s.as_bytes());
        parse_constellations_from_reader(rdr).expect("parse constellations")
    }

    #[test]
    fn parses_variable_length_rows_into_lines() {
        // 4 line segments for ORI
        let csv = "\
ORI,4.843611,8.9000,4.830833,6.9500,4.853611,5.6000,4.904167,2.4500,4.975833,1.7167,5.418889,6.3500,5.533611,-0.3000,5.408056,-2.3833,5.293333,-6.8500,5.242222,-8.2000,5.796111,-9.6667,5.679444,-1.9500,5.919444,7.4000,5.585556,9.9333,5.418889,6.3500
ORI,5.679444,-1.9500,5.603333,-1.2000,5.533611,-0.3000
ORI,5.919444,7.4000,6.039722,9.6500,6.126389,14.7667,5.906389,20.2667
ORI,6.039722,9.6500,6.198889,14.2167,6.065278,20.1333
";
        let res = parse_from_str(csv);
        assert_eq!(res.len(), 1);
        let ori = &res[0];

        // Basic constellation info/lookups
        assert_eq!(ori.name, "Orion");
        assert_eq!(ori.lines.len(), 4);

        // First row has 15 RA/Dec pairs
        assert_eq!(ori.lines[0].len(), 15);

        // Check RA hours->deg conversion on the first point: 4.843611h * 15 = 72.654165°
        let p0 = ori.lines[0][0];
        assert!(approx(p0.ra_deg, 72.654165, 1e-6));
        assert!(approx(p0.dec_deg, 8.9, 1e-4));

        // Row with 3 pairs
        assert_eq!(ori.lines[1].len(), 3);
        assert_eq!(ori.lines[3].len(), 3);
    }

    #[test]
    fn ignores_incomplete_trailing_field() {
        // One full pair (1.0,2.0) then a dangling RA (3.0) should ignore the last field.
        let csv = "ORI,1.0,2.0,3.0\n";
        let res = parse_from_str(csv);
        assert_eq!(res.len(), 1);
        let ori = &res[0];
        assert_eq!(ori.lines.len(), 1);
        assert_eq!(ori.lines[0].len(), 1);
        let p = ori.lines[0][0];
        assert!(approx(p.ra_deg, 15.0, 1e-12)); // 1h -> 15°
        assert!(approx(p.dec_deg, 2.0, 1e-12));
    }

    #[test]
    fn trims_whitespace_and_skips_blank_pairs() {
        // Contains spaces and a completely blank pair (,,) which should be skipped.
        let csv = "ORI, 6.0 , 1.0 , , , 7.0 , 2.0 \n";
        let res = parse_from_str(csv);
        let ori = &res[0];
        assert_eq!(ori.lines.len(), 1);
        assert_eq!(ori.lines[0].len(), 2);
        let p0 = ori.lines[0][0];
        let p1 = ori.lines[0][1];
        assert!(approx(p0.ra_deg, 90.0, 1e-12));
        assert!(approx(p0.dec_deg, 1.0, 1e-12));
        assert!(approx(p1.ra_deg, 105.0, 1e-12));
        assert!(approx(p1.dec_deg, 2.0, 1e-12));
    }
}
