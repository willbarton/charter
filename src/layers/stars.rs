use svg::node::element::{Circle, Group};

use crate::context::ChartContext;
use crate::geometry::{project, to_pixels};
use crate::layers::{group_with_class, Layer};

pub struct StarsLayer;
impl StarsLayer {
    pub fn new() -> Self {
        Self
    }
}

impl Layer for StarsLayer {
    fn render(&self, context: &ChartContext<'_>) -> Group {
        let mut g = group_with_class("stars");
        let scale = context.cfg.object_scale;

        for s in context.data.stars {
            if s.magnitude > context.cfg.limit_star_mag {
                continue;
            }
            if let Some(tp) = project(
                s.coords,
                context.cfg.center,
                context.cfg.projection,
                context.cfg.position_angle_deg,
            ) {
                let p = to_pixels(tp, context.layout.center_px, context.layout.scale);
                let r = (4.0 - 0.6 * s.magnitude).max(0.5) * scale;

                let c = Circle::new()
                    .set("id", s.identifier.as_str())
                    .set("class", "star")
                    .set("cx", p.x)
                    .set("cy", p.y)
                    .set("r", r);
                g = g.add(c);
            }
        }
        g
    }
}
