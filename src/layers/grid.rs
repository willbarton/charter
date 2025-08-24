use svg::node::element::path::Data;
use svg::node::element::{Group, Path};

use crate::context::ChartContext;
use crate::geometry::{sample_dec_parallel, sample_ra_meridian, split_segments};
use crate::layers::{group_with_class, Layer};

pub struct GridLayer;
impl GridLayer {
    pub fn new() -> Self {
        Self
    }
}

impl Layer for GridLayer {
    fn render(&self, context: &ChartContext<'_>) -> Group {
        let mut g = group_with_class("lines");
        let threshold = context.layout.split_threshold;

        // RA (hours)
        let mut ra_step_h = ((context.cfg.step_ra_deg as f64) / 15.0).round() as i32;
        if ra_step_h < 1 {
            ra_step_h = 1;
        }

        let mut h = 0;
        while h < 24 {
            let ra_deg = (h as f64) * 15.0;
            for seg in split_segments(&sample_ra_meridian(context, ra_deg, None), threshold) {
                if seg.len() < 2 {
                    continue;
                }
                let mut d = Data::new().move_to((seg[0].x, seg[0].y));
                for p in &seg[1..] {
                    d = d.line_to((p.x, p.y));
                }
                let path = Path::new()
                    .set("class", "graticule ra")
                    .set("fill", "none")
                    .set("d", d);
                g = g.add(path);
            }
            h += ra_step_h;
        }

        // Dec
        let step_dec = context.cfg.step_dec_deg as i32;
        let mut dec = -80;
        while dec <= 90 {
            for seg in split_segments(&sample_dec_parallel(context, dec as f64, None), threshold) {
                if seg.len() < 2 {
                    continue;
                }
                let mut d = Data::new().move_to((seg[0].x, seg[0].y));
                for p in &seg[1..] {
                    d = d.line_to((p.x, p.y));
                }
                let path = Path::new()
                    .set("class", "graticule dec")
                    .set("fill", "none")
                    .set("d", d);
                g = g.add(path);
            }
            dec += step_dec;
        }

        g
    }
}
