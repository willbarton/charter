use crate::config::ChartConfig;
use crate::context::{ChartContext, Datasets};
use crate::types::EQPoint;

// Check that the error between a and b is close enough
pub fn approx(a: f64, b: f64, eps: f64) -> bool {
    (a - b).abs() <= eps
}

pub fn make_context(patch: impl FnOnce(&mut ChartConfig)) -> ChartContext<'static> {
    let mut cfg = ChartConfig::default();
    cfg.center = EQPoint {
        ra_deg: 0.0,
        dec_deg: 0.0,
    };
    patch(&mut cfg);
    let data = Datasets {
        stars: &[],
        objects: &[],
        constellations: &[],
    };
    ChartContext::new(data, cfg)
}
