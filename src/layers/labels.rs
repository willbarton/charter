use svg::node::element::Group;

use crate::context::ChartContext;
use crate::geometry::{project, to_pixels};
use crate::layers::{group_with_class, text, Layer};
use crate::types::Point;

pub struct LabelsLayer {
    limit_star_label_mag: f64,
    limit_object_label_mag: f64,
    symbol_pad: f64,
    offsets: [(f64, f64); 6],
}
impl LabelsLayer {
    pub fn new() -> Self {
        Self {
            limit_star_label_mag: 1.0,
            limit_object_label_mag: 8.0,
            symbol_pad: 1.0,
            offsets: [
                (0.0, -10.0),
                (0.0, 10.0),
                (0.0, -16.0),
                (0.0, 16.0),
                (0.0, -20.0),
                (0.0, 20.0),
            ],
        }
    }
    fn boxes_overlap(a: (f64, f64, f64, f64), b: (f64, f64, f64, f64)) -> bool {
        let (ax, ay, aw, ah) = a;
        let (bx, by, bw, bh) = b;
        !(ax + aw <= bx || ax >= bx + bw || ay + ah <= by || ay >= by + bh)
    }
    fn label_box_centered(&self, x: f64, y_baseline: f64, text: &str) -> (f64, f64, f64, f64) {
        let ch = text.chars().count().max(2);
        let w = (ch as f64 * 7.0).max(16.0);
        let h = 12.0;
        let left = x - w / 2.0;
        let top = y_baseline - h;
        (left, top, w, h)
    }
    fn should_label(&self, kind: &str, mag: f64) -> bool {
        if kind.to_lowercase().contains("star") {
            mag <= self.limit_star_label_mag
        } else {
            mag <= self.limit_object_label_mag
        }
    }
    fn star_symbol_box(&self, p: Point, mag: f64) -> (f64, f64, f64, f64) {
        let mut r = (4.0 - 0.6 * mag).max(0.5);
        r += self.symbol_pad;
        (p.x - r, p.y - r, 2.0 * r, 2.0 * r)
    }
    fn object_symbol_box(&self, kind: &str, mag: f64, p: Point) -> (f64, f64, f64, f64) {
        let base = 10.0;
        let size = (base - mag).max(4.0);
        let pad = self.symbol_pad;
        match kind.to_lowercase().as_str() {
            "bright-nebula" => {
                let half = size / 2.0 + pad;
                (p.x - half, p.y - half, 2.0 * half, 2.0 * half)
            }
            "planetary-nebula" => {
                let half = size + pad;
                (p.x - half, p.y - half, 2.0 * half, 2.0 * half)
            }
            "galaxy" => {
                let rmax = size.max(size / 2.0) + pad;
                (p.x - rmax, p.y - rmax, 2.0 * rmax, 2.0 * rmax)
            }
            "open-cluster" | "globular-cluster" => {
                let r = size / 2.0 + pad;
                (p.x - r, p.y - r, 2.0 * r, 2.0 * r)
            }
            _ => {
                let half = size / 2.0 + pad;
                (p.x - half, p.y - half, 2.0 * half, 2.0 * half)
            }
        }
    }
    fn seed_symbol_boxes(&self, context: &ChartContext<'_>) -> Vec<(f64, f64, f64, f64)> {
        let mut boxes = Vec::new();
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
                boxes.push(self.star_symbol_box(p, s.magnitude));
            }
        }
        for o in context.data.objects {
            if o.magnitude > context.cfg.limit_object_mag {
                continue;
            }
            if let Some(tp) = project(
                o.coords,
                context.cfg.center,
                context.cfg.projection,
                context.cfg.position_angle_deg,
            ) {
                let p = to_pixels(tp, context.layout.center_px, context.layout.scale);
                boxes.push(self.object_symbol_box(&o.kind, o.magnitude, p));
            }
        }
        boxes
    }
}
impl Layer for LabelsLayer {
    fn render(&self, context: &ChartContext<'_>) -> Group {
        let mut g = group_with_class("labels");
        let mut placed = self.seed_symbol_boxes(context);

        // build candidates (brightest-first)
        #[derive(Clone)]
        struct Cand {
            magnitude: f64,
            is_star: bool,
            text: String,
            p: Point,
        }
        let mut cands: Vec<Cand> = Vec::new();

        for s in context.data.stars {
            if !self.should_label(&s.kind, s.magnitude) {
                continue;
            }
            if let Some(tp) = project(
                s.coords,
                context.cfg.center,
                context.cfg.projection,
                context.cfg.position_angle_deg,
            ) {
                let p = to_pixels(tp, context.layout.center_px, context.layout.scale);
                let text = if s.name.is_empty() {
                    format!("{} {}", s.catalog, s.identifier)
                } else {
                    s.name.clone()
                };
                cands.push(Cand {
                    magnitude: s.magnitude,
                    is_star: true,
                    text,
                    p,
                });
            }
        }
        for o in context.data.objects {
            // Messier object labels always fall through to positioning
            if o.catalog != "M" && !self.should_label(&o.kind, o.magnitude) {
                continue;
            }
            if let Some(tp) = project(
                o.coords,
                context.cfg.center,
                context.cfg.projection,
                context.cfg.position_angle_deg,
            ) {
                let p = to_pixels(tp, context.layout.center_px, context.layout.scale);
                let text = if o.name.is_empty() {
                    format!("{} {}", o.catalog, o.identifier)
                } else {
                    o.name.clone()
                };
                cands.push(Cand {
                    magnitude: o.magnitude,
                    is_star: false,
                    text,
                    p,
                });
            }
        }
        cands.sort_by(|a, b| a.magnitude.partial_cmp(&b.magnitude).unwrap());

        let l = &context.layout;
        let (left, top) = (l.plot_x, l.plot_y);
        let (right, bottom) = (left + l.plot_w, top + l.plot_h);

        for c in cands {
            let cls = if c.is_star {
                "star-label"
            } else {
                "object-label"
            };
            for (dx, dy) in self.offsets {
                let ax = c.p.x + dx;
                let ay = c.p.y + dy;

                let (bx, by, bw, bh) = self.label_box_centered(ax, ay, &c.text);
                if bx < left || bx + bw > right || by < top || by + bh > bottom {
                    continue;
                }
                if placed
                    .iter()
                    .any(|&b| Self::boxes_overlap((bx, by, bw, bh), b))
                {
                    continue;
                }

                placed.push((bx, by, bw, bh));
                g = g.add(text(cls, ax, by + bh, "middle", &c.text));
                break;
            }
        }

        g
    }
}
