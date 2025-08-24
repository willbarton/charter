use svg::node::element::Group as G;
use svg::node::element::{Circle, Ellipse, Group, Line, Rectangle};

use crate::context::ChartContext;
use crate::geometry::{project, to_pixels};
use crate::layers::{group_with_class, Layer};

fn r_mag(mag: f64, r_min: f64, r_max: f64, mag_bright: f64, mag_faint: f64) -> f64 {
    let m = mag.clamp(mag_bright, mag_faint);
    let f = 10f64.powf(-0.4 * m);
    let fb = 10f64.powf(-0.4 * mag_bright);
    let ff = 10f64.powf(-0.4 * mag_faint);
    let t = (f - ff) / (fb - ff);
    r_min + (r_max - r_min) * t
}

fn r_size(arcmin: f64, k: f64, alpha: f64, cap: f64) -> f64 {
    if arcmin <= 0.0 {
        0.0
    } else {
        (k * arcmin.powf(alpha)).min(cap)
    }
}

fn radius(mag: f64, arcmin: Option<f64>, w_mag: f64, w_size: f64, floor: f64) -> f64 {
    let by_mag = r_mag(mag, 4.0, 18.0, -1.0, 10.0);
    let by_size = r_size(arcmin.unwrap_or(0.0), 1.2, 0.5, 16.0);
    (w_mag * by_mag + w_size * by_size).max(floor)
}

pub struct ObjectsLayer;
impl ObjectsLayer {
    pub fn new() -> Self {
        Self
    }
}

impl Layer for ObjectsLayer {
    fn render(&self, context: &ChartContext<'_>) -> Group {
        let mut g = group_with_class("objects");
        let scale = context.cfg.object_scale;

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
                let id = &o.identifier;
                let kind = o.kind.as_str();

                match kind {
                    "open-cluster" => {
                        let size = radius(o.magnitude, Some(o.size.major), 1.0, 0.3, 6.0) * scale;
                        let r = size * 0.5;
                        g = g.add(
                            Circle::new()
                                .set("id", id.as_str())
                                .set("class", "open-cluster object")
                                .set("cx", p.x)
                                .set("cy", p.y)
                                .set("r", r),
                        );
                    }
                    "globular-cluster" => {
                        let size = radius(o.magnitude, Some(o.size.major), 1.0, 0.3, 6.0) * scale;
                        let r = size * 0.5;
                        let mut gg = G::new()
                            .set("id", id.as_str())
                            .set("class", "globular-cluster object");
                        gg = gg.add(Circle::new().set("cx", p.x).set("cy", p.y).set("r", r));
                        gg = gg.add(
                            Line::new()
                                .set("x1", p.x - r)
                                .set("y1", p.y)
                                .set("x2", p.x + r)
                                .set("y2", p.y),
                        );
                        gg = gg.add(
                            Line::new()
                                .set("x1", p.x)
                                .set("y1", p.y - r)
                                .set("x2", p.x)
                                .set("y2", p.y + r),
                        );
                        g = g.add(gg);
                    }
                    "bright-nebula" => {
                        let size = radius(o.magnitude, Some(o.size.major), 1.0, 0.3, 6.0) * scale;
                        let half = size * 0.5;
                        g = g.add(
                            Rectangle::new()
                                .set("id", id.as_str())
                                .set("class", "bright-nebula object")
                                .set("x", p.x - half)
                                .set("y", p.y - half)
                                .set("width", 2.0 * half)
                                .set("height", 2.0 * half),
                        );
                    }
                    "galaxy" => {
                        let size = r_mag(o.magnitude, 4.0, 18.0, -1.0, 10.0) * scale;
                        let rx = size * 0.7;
                        let ry = size * 0.35;
                        let gg = G::new()
                            .set("id", id.as_str())
                            .set("class", "galaxy object")
                            .set(
                                "transform",
                                format!("rotate({:.2},{:.2},{:.2})", o.angle, p.x, p.y),
                            )
                            .add(
                                Ellipse::new()
                                    .set("cx", p.x)
                                    .set("cy", p.y)
                                    .set("rx", rx)
                                    .set("ry", ry),
                            );
                        g = g.add(gg);
                    }
                    "planetary-nebula" => {
                        let size = radius(o.magnitude, Some(o.size.major), 1.0, 0.3, 6.0) * scale;
                        let r = size / 4.0;
                        let cross = size / 2.0;
                        let mut gg = G::new()
                            .set("id", id.as_str())
                            .set("class", "planetary-nebula object");
                        gg = gg.add(Circle::new().set("cx", p.x).set("cy", p.y).set("r", r));
                        gg = gg.add(
                            Line::new()
                                .set("x1", p.x - cross)
                                .set("y1", p.y)
                                .set("x2", p.x + cross)
                                .set("y2", p.y),
                        );
                        gg = gg.add(
                            Line::new()
                                .set("x1", p.x)
                                .set("y1", p.y - cross)
                                .set("x2", p.x)
                                .set("y2", p.y + cross),
                        );
                        g = g.add(gg);
                    }
                    _ => {
                        let size = r_mag(o.magnitude, 4.0, 18.0, -1.0, 10.0) * scale;
                        let half = size * 0.5;
                        let mut gg = G::new().set("id", id.as_str()).set("class", "object");
                        gg = gg.add(
                            Line::new()
                                .set("x1", p.x - half)
                                .set("y1", p.y)
                                .set("x2", p.x + half)
                                .set("y2", p.y),
                        );
                        gg = gg.add(
                            Line::new()
                                .set("x1", p.x)
                                .set("y1", p.y - half)
                                .set("x2", p.x)
                                .set("y2", p.y + half),
                        );
                        g = g.add(gg);
                    }
                }
            }
        }

        g
    }
}
