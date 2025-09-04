mod chart;
mod config;
mod context;
mod data;
mod geometry;
mod layers;
mod layout;
mod types;

use crate::chart::Chart;
use crate::config::{ChartConfig, Margin};
use crate::context::Datasets;
use crate::data::{load_constellations, load_objects, load_stars};
use crate::types::{
    hours_to_degrees, parse_dms, parse_hms, sexagesimal_dms_to_degrees, sexagesimal_hms_to_hours,
    EQPoint, Projection,
};

use anyhow::{anyhow, Context, Result};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "charter")]
#[command(about = "Simple and attractive star charts", version)]
struct Args {
    /// Center RA either as hour:minute:second (e.g. "5:35:17.3") or decimal degrees ("83.821")
    #[arg(long = "ra", alias = "center-ra")]
    ra: String,

    /// Center Dec as degree:minute:second (e.g. "-5:23:28") or decimal degrees ("-5.391")
    #[arg(long = "dec", alias = "center-dec")]
    dec: String,

    /// Field of view (in degrees)
    #[arg(long, default_value_t = 40.0)]
    fov: f64,

    /// Type of projectionto draw, either gnomonic, stereographic, spherical, or altaz
    #[arg(long, default_value = "gnomonic")]
    projection: String,

    /// Dimmest magnitude stars to draw
    #[arg(long, default_value_t = 6.5)]
    limit_star_mag: f64,

    /// Dimmest magnitude objects to draw
    #[arg(long, default_value_t = 10.0)]
    limit_object_mag: f64,

    /// Scale multiplier for object symbol size relative to its physical size and/or magnitude
    #[arg(long, default_value_t = 1.25)]
    object_scale: f64,

    /// Output SVG path
    #[arg(short = 'o', long = "out")]
    out: String,

    /// Optional CSS override file path; if omitted embedded CSS is used
    #[arg(long)]
    css: Option<String>,

    /// Output image width in pixels
    #[arg(long, default_value_t = 600)]
    width: u32,

    /// Output image height in pixels
    #[arg(long, default_value_t = 800)]
    height: u32,

    /// RA gridlines step in degrees (e.g., 15)
    #[arg(long, default_value_t = 15)]
    step_ra_deg: u32,

    /// Dec gridlines step in degrees (e.g., 10)
    #[arg(long, default_value_t = 10)]
    step_dec_deg: u32,

    /// Optional path override for stars (HYG format expected)
    #[arg(long)]
    hyg_path: Option<String>,

    /// Optional path override for deep-sky objects (OpenNGC format expected)
    #[arg(long)]
    ngc_path: Option<String>,

    /// Optional path override for constellations vectors CSV
    #[arg(long)]
    constellations_path: Option<String>,
}

fn parse_ra_deg(s: &str) -> Result<f64> {
    if s.contains(':') {
        let (h, m, sec) = parse_hms(s).ok_or_else(|| anyhow!("bad RA HMS: {s}"))?;
        let hours = sexagesimal_hms_to_hours(h, m, sec);
        Ok(hours_to_degrees(hours).rem_euclid(360.0))
    } else {
        let deg: f64 = s.parse().context("RA must be HMS or degrees")?;
        Ok(deg.rem_euclid(360.0))
    }
}

fn parse_dec_deg(s: &str) -> Result<f64> {
    if s.contains(':') {
        let (d, m, sec) = parse_dms(s).ok_or_else(|| anyhow!("bad Dec DMS: {s}"))?;
        Ok(sexagesimal_dms_to_degrees(d, m, sec))
    } else {
        let deg: f64 = s.parse().context("Dec must be DMS or degrees")?;
        Ok(deg)
    }
}

fn parse_projection(s: &str) -> Result<Projection> {
    Projection::from_str(&s.to_lowercase()).ok_or_else(|| {
        anyhow!("invalid projection '{s}'. Use: gnomonic | stereographic | spherical | altaz")
    })
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let ra_deg = parse_ra_deg(&args.ra)?;
    let dec_deg = parse_dec_deg(&args.dec)?;
    let center = EQPoint { ra_deg, dec_deg };
    let projection = parse_projection(&args.projection)?;

    let stars = load_stars(args.hyg_path.as_deref())?;
    let objects = load_objects(args.ngc_path.as_deref())?;
    let constellations = load_constellations(args.constellations_path.as_deref())?;

    let cfg = ChartConfig {
        center,
        position_angle_deg: 0.0,
        projection,
        fov_deg: args.fov,
        width: args.width,
        height: args.height,
        margin: Margin::uniform(40),
        step_ra_deg: args.step_ra_deg,
        step_dec_deg: args.step_dec_deg,
        limit_star_mag: args.limit_star_mag,
        limit_object_mag: args.limit_object_mag,
        object_scale: args.object_scale,
    };

    let data = Datasets {
        stars: &stars,
        objects: &objects,
        constellations: &constellations,
    };

    let chart = Chart::new(data, cfg, args.css);
    chart
        .to_file(&args.out)
        .with_context(|| format!("writing {}", args.out))?;

    Ok(())
}

#[cfg(test)]
mod test_utils;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::approx;

    #[test]
    fn ra_hms_parses_and_normalizes() {
        // 5:35:17.3 -> 5.588138... hours -> 83.822083... deg
        let ra = parse_ra_deg("5:35:17.3").unwrap();
        assert!(approx(ra, 83.82208333333332, 1e-9));

        // 24:00:00 -> 360 deg -> normalized to 0
        let ra = parse_ra_deg("24:00:00").unwrap();
        assert!(approx(ra, 0.0, 1e-12));

        // Negative degrees should wrap into [0, 360)
        let ra = parse_ra_deg("-30").unwrap();
        assert!(approx(ra, 330.0, 1e-12));
    }

    #[test]
    fn ra_degrees_parses_direct() {
        let ra = parse_ra_deg("83.82208333333332").unwrap();
        assert!(approx(ra, 83.82208333333332, 1e-12));

        // >360 wraps
        let ra = parse_ra_deg("720").unwrap();
        assert!(approx(ra, 0.0, 1e-12));
    }

    #[test]
    fn ra_bad_input_errors() {
        // Completely non-numeric degrees is an error
        assert!(parse_ra_deg("not-a-number").is_err());
        // Wrong HMS arity (needs exactly 3 fields)
        assert!(parse_ra_deg("1:2").is_err());
    }

    #[test]
    fn dec_dms_parses_with_sign() {
        // -5:23:28 -> -5.391111... deg
        let dec = parse_dec_deg("-5:23:28").unwrap();
        assert!(approx(dec, -5.391111111111111, 1e-9));

        // +10:00:00 -> 10 deg
        let dec = parse_dec_deg("+10:00:00").unwrap();
        assert!(approx(dec, 10.0, 1e-12));
    }

    #[test]
    fn dec_degrees_parses_direct() {
        let dec = parse_dec_deg("-5.3911111111").unwrap();
        assert!(approx(dec, -5.3911111111, 1e-12));
    }

    #[test]
    fn dec_bad_input_errors() {
        assert!(parse_dec_deg("bad").is_err());
        assert!(parse_dec_deg("1:2").is_err()); // not DMS (needs 3 fields)
    }

    #[test]
    fn projection_parses_case_insensitive() {
        assert!(matches!(
            parse_projection("gnomonic").unwrap(),
            Projection::Gnomonic
        ));
        assert!(matches!(
            parse_projection("Stereographic").unwrap(),
            Projection::Stereographic
        ));
        assert!(matches!(
            parse_projection("SPHERICAL").unwrap(),
            Projection::Spherical
        ));
        assert!(matches!(
            parse_projection("AltAz").unwrap(),
            Projection::AltAz
        ));
    }

    #[test]
    fn projection_invalid_errors() {
        assert!(parse_projection("unknown").is_err());
    }
}
