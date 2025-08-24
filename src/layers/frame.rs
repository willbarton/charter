use std::collections::HashSet;
use svg::node::element::{Group, Line, Rectangle};

use crate::context::ChartContext;
use crate::geometry::{sample_dec_parallel, sample_ra_meridian, split_segments};
use crate::layers::{group_with_class, text, Layer};
use crate::types::Point;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Clone)]
struct Mark {
    x: f64,
    y: f64,
    side: Side,
    label: String,
}

fn dedup_marks(marks: Vec<Mark>) -> Vec<Mark> {
    let mut out = Vec::new();
    let mut seen: HashSet<(Side, i32, i32, String)> = HashSet::new();
    for m in marks {
        let key = (
            m.side,
            (m.x * 10.0).round() as i32,
            (m.y * 10.0).round() as i32,
            m.label.clone(),
        );
        if seen.insert(key) {
            out.push(m);
        }
    }
    out
}

fn edge_hits(
    poly: &[Point],
    want: &[Side],
    top: f64,
    bottom: f64,
    left: f64,
    right: f64,
) -> Vec<Mark> {
    let mut hits = Vec::new();
    let wt = want.contains(&Side::Top);
    let wb = want.contains(&Side::Bottom);
    let wl = want.contains(&Side::Left);
    let wr = want.contains(&Side::Right);

    for w in poly.windows(2) {
        let (a, b) = (w[0], w[1]);
        let (x1, y1, x2, y2) = (a.x, a.y, b.x, b.y);
        let (dx, dy) = (x2 - x1, y2 - y1);

        if wt && dy != 0.0 && (y1 - top) * (y2 - top) <= 0.0 {
            let t = (top - y1) / dy;
            if (0.0..=1.0).contains(&t) {
                let x = x1 + t * dx;
                if x >= left - 1e-6 && x <= right + 1e-6 {
                    hits.push(Mark {
                        x,
                        y: top,
                        side: Side::Top,
                        label: String::new(),
                    });
                }
            }
        }
        if wb && dy != 0.0 && (y1 - bottom) * (y2 - bottom) <= 0.0 {
            let t = (bottom - y1) / dy;
            if (0.0..=1.0).contains(&t) {
                let x = x1 + t * dx;
                if x >= left - 1e-6 && x <= right + 1e-6 {
                    hits.push(Mark {
                        x,
                        y: bottom,
                        side: Side::Bottom,
                        label: String::new(),
                    });
                }
            }
        }
        if wl && dx != 0.0 && (x1 - left) * (x2 - left) <= 0.0 {
            let t = (left - x1) / dx;
            if (0.0..=1.0).contains(&t) {
                let y = y1 + t * dy;
                if y >= top - 1e-6 && y <= bottom + 1e-6 {
                    hits.push(Mark {
                        x: left,
                        y,
                        side: Side::Left,
                        label: String::new(),
                    });
                }
            }
        }
        if wr && dx != 0.0 && (x1 - right) * (x2 - right) <= 0.0 {
            let t = (right - x1) / dx;
            if (0.0..=1.0).contains(&t) {
                let y = y1 + t * dy;
                if y >= top - 1e-6 && y <= bottom + 1e-6 {
                    hits.push(Mark {
                        x: right,
                        y,
                        side: Side::Right,
                        label: String::new(),
                    });
                }
            }
        }
    }
    hits
}

pub struct FrameLayer {
    fine_step_ra_deg: f64,
    fine_step_dec_deg: i32,
}
impl FrameLayer {
    pub fn new() -> Self {
        Self {
            fine_step_ra_deg: 3.75,
            fine_step_dec_deg: 2,
        }
    }
}

impl Layer for FrameLayer {
    fn render(&self, context: &ChartContext<'_>) -> Group {
        let mut g = group_with_class("frame");
        let l = &context.layout;
        let (px, py, pw, ph) = (l.plot_x, l.plot_y, l.plot_w, l.plot_h);
        let (top, bottom, left, right) = (py, py + ph, px, px + pw);

        // Border rectangle
        g = g.add(
            Rectangle::new()
                .set("x", px)
                .set("y", py)
                .set("width", pw)
                .set("height", ph)
                .set("fill", "none")
                .set("stroke", "black")
                .set("class", "border"),
        );

        // RA ticks (top/bottom)
        let mut ra_marks: Vec<Mark> = Vec::new();
        let ra_step_h = self.fine_step_ra_deg / 15.0;
        let n = (24.0 / ra_step_h).floor() as usize;
        let is_major = |ra_deg: f64| -> bool {
            let step = context.cfg.step_ra_deg as f64;
            let k = (ra_deg / step).round();
            (ra_deg - k * step).abs() < 1e-8
        };

        for i in 0..n {
            let h = i as f64 * ra_step_h;
            let ra_deg = h * 15.0;

            let pts = sample_ra_meridian(context, ra_deg, None);
            for seg in split_segments(&pts, l.split_threshold) {
                for mut m in edge_hits(&seg, &[Side::Top, Side::Bottom], top, bottom, left, right) {
                    if is_major(ra_deg) {
                        m.label = format!("{:.0}h", h.round());
                        ra_marks.push(m.clone());
                    }
                    ra_marks.push(m);
                }
            }
        }

        for m in dedup_marks(ra_marks) {
            match m.side {
                Side::Top => {
                    let len = if m.label.is_empty() { 3.0 } else { 6.0 };
                    g = g.add(
                        Line::new()
                            .set("x1", m.x)
                            .set("y1", top)
                            .set("x2", m.x)
                            .set("y2", top - len)
                            .set("class", "tick"),
                    );
                    if !m.label.is_empty() {
                        g = g.add(text("tick-label", m.x, top - 10.0, "middle", &m.label));
                    }
                }
                Side::Bottom => {
                    let len = 6.0;
                    g = g.add(
                        Line::new()
                            .set("x1", m.x)
                            .set("y1", bottom)
                            .set("x2", m.x)
                            .set("y2", bottom + len)
                            .set("class", "tick"),
                    );
                    if !m.label.is_empty() {
                        g = g.add(text("tick-label", m.x, bottom + 20.0, "middle", &m.label));
                    }
                }
                _ => {}
            }
        }

        // Dec ticks (left/right)
        let mut dec_marks: Vec<Mark> = Vec::new();
        for d in (-80..=90).step_by(self.fine_step_dec_deg as usize) {
            let pts = sample_dec_parallel(context, d as f64, None);
            for seg in split_segments(&pts, l.split_threshold) {
                for mut m in edge_hits(&seg, &[Side::Left, Side::Right], top, bottom, left, right) {
                    if d % (context.cfg.step_dec_deg as i32) == 0 {
                        m.label = format!("{d}°");
                        dec_marks.push(m.clone());
                    }
                    dec_marks.push(m);
                }
            }
        }

        for m in dedup_marks(dec_marks) {
            match m.side {
                Side::Left => {
                    let len = if m.label.is_empty() { 3.0 } else { 6.0 };
                    g = g.add(
                        Line::new()
                            .set("x1", left)
                            .set("y1", m.y)
                            .set("x2", left - len)
                            .set("y2", m.y)
                            .set("class", "tick"),
                    );
                    if !m.label.is_empty() {
                        g = g.add(text("tick-label", left - 10.0, m.y + 4.0, "end", &m.label));
                    }
                }
                Side::Right => {
                    let len = if m.label.is_empty() { 3.0 } else { 6.0 };
                    g = g.add(
                        Line::new()
                            .set("x1", right)
                            .set("y1", m.y)
                            .set("x2", right + len)
                            .set("y2", m.y)
                            .set("class", "tick"),
                    );
                    if !m.label.is_empty() {
                        g = g.add(text(
                            "tick-label",
                            right + 10.0,
                            m.y + 4.0,
                            "start",
                            &m.label,
                        ));
                    }
                }
                _ => {}
            }
        }

        g
    }
}
