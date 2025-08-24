use svg::node::element::{Group, Line};

use crate::context::ChartContext;
use crate::geometry::{project, to_pixels};
use crate::layers::{group_with_class, Layer};

pub struct ZenithLayer;
impl ZenithLayer {
    pub fn new() -> Self {
        Self
    }
}

impl Layer for ZenithLayer {
    fn render(&self, context: &ChartContext<'_>) -> Group {
        let mut g = group_with_class("zenith");

        if let Some(tp) = project(
            context.cfg.center,
            context.cfg.center,
            context.cfg.projection,
            context.cfg.position_angle_deg,
        ) {
            let p = to_pixels(tp, context.layout.center_px, context.layout.scale);
            let size = 10.0;

            let h = Line::new()
                .set("x1", p.x - size / 2.0)
                .set("y1", p.y)
                .set("x2", p.x + size / 2.0)
                .set("y2", p.y)
                .set("stroke-width", 2);
            let v = Line::new()
                .set("x1", p.x)
                .set("y1", p.y - size / 2.0)
                .set("x2", p.x)
                .set("y2", p.y + size / 2.0)
                .set("stroke-width", 2);

            g = g.add(h).add(v);
        }
        g
    }
}
