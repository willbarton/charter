use svg::node::element::path::Data;
use svg::node::element::{Group, Path};

use crate::context::ChartContext;
use crate::geometry::{project, split_segments, to_pixels};
use crate::layers::{group_with_class, Layer};
use crate::types::EQPoint;

pub struct EclipticLayer;
impl EclipticLayer {
    pub fn new() -> Self {
        Self
    }
}

impl Layer for EclipticLayer {
    fn render(&self, context: &ChartContext<'_>) -> Group {
        let mut g = group_with_class("ecliptic");
        let eps = 23.43928_f64.to_radians();

        // sample longitudes 0..360 step 2°
        let mut pts = Vec::new();
        let mut lon_deg = 0usize;
        while lon_deg <= 360 {
            let lon = (lon_deg as f64).to_radians();
            let dec = (lon.sin() * eps.sin()).asin();
            let ra = (lon.sin() * eps.cos()).atan2(lon.cos());

            let eq = EQPoint {
                ra_deg: ra.to_degrees().rem_euclid(360.0),
                dec_deg: dec.to_degrees(),
            };
            if let Some(tp) = project(
                eq,
                context.cfg.center,
                context.cfg.projection,
                context.cfg.position_angle_deg,
            ) {
                pts.push(to_pixels(
                    tp,
                    context.layout.center_px,
                    context.layout.scale,
                ));
            }
            lon_deg += 2;
        }

        for seg in split_segments(&pts, context.layout.split_threshold)
            .into_iter()
            .filter(|s| s.len() >= 2)
        {
            let mut d = Data::new().move_to((seg[0].x, seg[0].y));
            for p in &seg[1..] {
                d = d.line_to((p.x, p.y));
            }
            let path = Path::new()
                .set("class", "ecliptic")
                .set("fill", "none")
                .set("d", d);
            g = g.add(path);
        }

        g
    }
}
