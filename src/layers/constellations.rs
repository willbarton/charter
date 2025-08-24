use svg::node::element::path::Data;
use svg::node::element::{Group, Path, Text};

use crate::context::ChartContext;
use crate::geometry::{project, split_segments, to_pixels};
use crate::layers::{group_with_class, Layer};

pub struct ConstellationsLayer;
impl ConstellationsLayer {
    pub fn new() -> Self {
        Self
    }
}

impl Layer for ConstellationsLayer {
    fn render(&self, context: &ChartContext<'_>) -> Group {
        let mut g = group_with_class("constellations");
        let threshold = context.layout.split_threshold;

        for c in context.data.constellations {
            let mut all_pts: Vec<crate::types::Point> = Vec::new();
            for line in &c.lines {
                let mut pts = Vec::with_capacity(line.len());
                for &eq in line {
                    if let Some(tp) = project(
                        eq,
                        context.cfg.center,
                        context.cfg.projection,
                        context.cfg.position_angle_deg,
                    ) {
                        let p = to_pixels(tp, context.layout.center_px, context.layout.scale);
                        pts.push(p);
                        all_pts.push(p);
                    }
                }
                // segment & emit paths
                for seg in split_segments(&pts, threshold)
                    .into_iter()
                    .filter(|s| s.len() >= 2)
                {
                    let mut d = Data::new().move_to((seg[0].x, seg[0].y));
                    for p in &seg[1..] {
                        d = d.line_to((p.x, p.y));
                    }
                    let path = Path::new()
                        .set("class", "constellation")
                        .set("fill", "none")
                        .set("d", d);
                    g = g.add(path);
                }
            }

            // Add a label at the bbox center of visible points
            if all_pts.len() >= 2 {
                let (mut min_x, mut max_x) = (f64::INFINITY, f64::NEG_INFINITY);
                let (mut min_y, mut max_y) = (f64::INFINITY, f64::NEG_INFINITY);
                for p in &all_pts {
                    if p.x < min_x {
                        min_x = p.x;
                    }
                    if p.x > max_x {
                        max_x = p.x;
                    }
                    if p.y < min_y {
                        min_y = p.y;
                    }
                    if p.y > max_y {
                        max_y = p.y;
                    }
                }
                let (cx, cy) = ((min_x + max_x) * 0.5, (min_y + max_y) * 0.5);
                let label: Text = svg::node::element::Text::new(&c.name)
                    .set("class", "constellation-label")
                    .set("x", cx)
                    .set("y", cy)
                    .set("text-anchor", "middle")
                    .set("dominant-baseline", "middle");
                g = g.add(label);
            }
        }
        g
    }
}
