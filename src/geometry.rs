use crate::context::ChartContext;
use crate::types::{EQPoint, Point, Projection};
use std::f64::consts::PI;

#[inline]
fn clamp(x: f64, lo: f64, hi: f64) -> f64 {
    if x < lo {
        lo
    } else if x > hi {
        hi
    } else {
        x
    }
}

#[inline]
pub fn to_pixels(tp: Point, center_px: Point, scale: f64) -> Point {
    Point {
        x: center_px.x + tp.x * scale,
        y: center_px.y - tp.y * scale,
    }
}

// Project an equatorial point relative to a chart center.
// - `coords` / `center`: RA/Dec in **degrees**
// - `projection`: which chart projection to use
// - `position_angle_deg`: rotate so PA=0 has +y to north; positive PA rotates the chart counterclockwise
//
// Returns `None` when the point is on the “back” side of the sphere for all
// projections **except** stereographic (which allows it).
pub fn project(
    coords: EQPoint,
    center: EQPoint,
    projection: Projection,
    position_angle_deg: f64,
) -> Option<Point> {
    // deg -> rad
    let ra = coords.ra_deg.to_radians();
    let dec = coords.dec_deg.to_radians();
    let cra = center.ra_deg.to_radians();
    let cde = center.dec_deg.to_radians();

    // relative RA, normalized to [-PI, PI] (robust near RA=0)
    let d_ra = ra - cra;

    // Spherical law of cosines: cos(zenith)
    let cos_z = clamp(
        cde.sin() * dec.sin() + cde.cos() * dec.cos() * d_ra.cos(),
        -1.0,
        1.0,
    );
    let zenith = cos_z.acos();

    // Azimuth (bearing from the center), minus the position angle
    let y = d_ra.sin() * dec.cos();
    let x = cde.cos() * dec.sin() - cde.sin() * dec.cos() * d_ra.cos();
    let az = y.atan2(x) - position_angle_deg.to_radians();

    // If behind the horizon and not stereographic, drop it.
    if zenith > PI / 2.0 && !matches!(projection, Projection::Stereographic) {
        return None;
    }

    // Radial mapping by projection
    let r = match projection {
        Projection::Gnomonic => zenith.tan(),
        Projection::Stereographic => (zenith / 2.0).tan(),
        Projection::Spherical => zenith.sin(),
        Projection::AltAz => zenith / (PI / 2.0),
    };

    Some(Point {
        x: -r * az.sin(),
        y: r * az.cos(),
    })
}

pub fn split_segments(points: &[Point], threshold: f64) -> Vec<Vec<Point>> {
    if points.is_empty() {
        return vec![];
    }
    let mut segs = Vec::new();
    let mut seg = vec![points[0]];
    for w in points.windows(2) {
        let (a, b) = (w[0], w[1]);
        if (b.x - a.x).abs() > threshold || (b.y - a.y).abs() > threshold {
            segs.push(seg);
            seg = vec![b];
        } else {
            seg.push(b);
        }
    }
    if !seg.is_empty() {
        segs.push(seg);
    }
    segs
}

pub fn sample_ra_meridian(
    context: &ChartContext<'_>,
    ra_deg: f64,
    step_opt: Option<u32>,
) -> Vec<Point> {
    let step = step_opt.unwrap_or_else(|| context.adaptive_step_deg());
    let ra = ra_deg.rem_euclid(360.0);
    let mut out = Vec::new();
    let mut d = -90;
    while d <= 90 {
        if let Some(tp) = project(
            EQPoint {
                ra_deg: ra,
                dec_deg: d as f64,
            },
            context.cfg.center,
            context.cfg.projection,
            context.cfg.position_angle_deg,
        ) {
            out.push(to_pixels(
                tp,
                context.layout.center_px,
                context.layout.scale,
            ));
        }
        d += step as i32;
    }
    out
}

pub fn sample_dec_parallel(
    context: &ChartContext<'_>,
    dec_deg: f64,
    step_opt: Option<u32>,
) -> Vec<Point> {
    let step = step_opt.unwrap_or_else(|| context.adaptive_step_deg());
    let mut out = Vec::new();
    let mut r = 0;
    while r < 360 {
        if let Some(tp) = project(
            EQPoint {
                ra_deg: r as f64,
                dec_deg,
            },
            context.cfg.center,
            context.cfg.projection,
            context.cfg.position_angle_deg,
        ) {
            out.push(to_pixels(
                tp,
                context.layout.center_px,
                context.layout.scale,
            ));
        }
        r += step as i32;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{approx, make_context};
    use crate::types::{EQPoint, Point, Projection};

    #[test]
    fn center_projects_to_origin() {
        let c = EQPoint {
            ra_deg: 0.0,
            dec_deg: 0.0,
        };
        let p = project(c, c, Projection::Gnomonic, 0.0).unwrap();
        assert!(approx(p.x, 0.0, 1e-15));
        assert!(approx(p.y, 0.0, 1e-15));
    }

    #[test]
    fn small_offset_gnomonic_equator_east_is_negative_x() {
        // 1° east (RA +1°, Dec 0) from an equatorial center
        let c = EQPoint {
            ra_deg: 0.0,
            dec_deg: 0.0,
        };
        let s = EQPoint {
            ra_deg: 1.0,
            dec_deg: 0.0,
        };
        let p = project(s, c, Projection::Gnomonic, 0.0).unwrap();
        // For this geometry, az ≈ +90°, so (x,y) ≈ (-tan(1°), 0)
        assert!(approx(p.x, -(1.0_f64.to_radians().tan()), 1e-12));
        assert!(approx(p.y, 0.0, 1e-12));
    }

    #[test]
    fn position_angle_rotates_counterclockwise() {
        // Same as above but PA = 90°; az -> az - 90°, point rotates to +y axis
        let c = EQPoint {
            ra_deg: 0.0,
            dec_deg: 0.0,
        };
        let s = EQPoint {
            ra_deg: 1.0,
            dec_deg: 0.0,
        };
        let p = project(s, c, Projection::Gnomonic, 90.0).unwrap();
        assert!(approx(p.x, 0.0, 1e-12));
        assert!(approx(p.y, 1.0_f64.to_radians().tan(), 1e-12));
    }

    #[test]
    fn gnomonic_drops_backside_but_stereographic_keeps_it() {
        // 120° away on the equator => zenith = 120° (>90°)
        let c = EQPoint {
            ra_deg: 0.0,
            dec_deg: 0.0,
        };
        let s = EQPoint {
            ra_deg: 120.0,
            dec_deg: 0.0,
        };

        // Gnomonic returns None
        assert!(project(s, c, Projection::Gnomonic, 0.0).is_none());

        // Stereographic returns Some with r = tan(zenith/2) = tan(60°) = √3
        let p = project(s, c, Projection::Stereographic, 0.0).unwrap();
        assert!(approx(p.x, -(60.0_f64.to_radians().tan()), 1e-12)); // ≈ -√3
        assert!(approx(p.y, 0.0, 1e-12));
    }

    #[test]
    fn ra_wrap_equivalent_delta_produces_same_tangent_point() {
        // Case A: center 359°, star 1° → ΔRA = -358° ≡ +2°
        let p1 = super::project(
            EQPoint {
                ra_deg: 1.0,
                dec_deg: 0.0,
            },
            EQPoint {
                ra_deg: 359.0,
                dec_deg: 0.0,
            },
            Projection::Gnomonic,
            0.0,
        )
        .unwrap();

        // Case B: center 1°, star 3° → ΔRA = +2°
        let p2 = super::project(
            EQPoint {
                ra_deg: 3.0,
                dec_deg: 0.0,
            },
            EQPoint {
                ra_deg: 1.0,
                dec_deg: 0.0,
            },
            Projection::Gnomonic,
            0.0,
        )
        .unwrap();

        assert!((p1.x - p2.x).abs() <= 1e-12);
        assert!((p1.y - p2.y).abs() <= 1e-12);
    }

    #[test]
    fn to_pixels_applies_center_and_scale_with_y_flip() {
        // Prepare a simple context to get scale and center
        let context = make_context(|_| {});
        let l = context.layout;

        // Take a known tangent-plane point: (-tan 1°, 0)
        let r = (1.0_f64.to_radians()).tan();
        let tp = Point { x: -r, y: 0.0 };
        let px = super::to_pixels(tp, l.center_px, l.scale);
        assert!(approx(px.x, l.center_px.x - r * l.scale, 1e-10));
        assert!(approx(px.y, l.center_px.y, 1e-10));
    }

    #[test]
    fn sample_ra_meridian_returns_vertical_axis_pixels() {
        // Use stereographic so Dec = ±90° are finite (r = tan(90°/2) = 1)
        let context = make_context(|cfg| cfg.projection = Projection::Stereographic);

        // Deterministic step: -90..90 inclusive at 30° → 7 samples
        let pts = super::sample_ra_meridian(&context, 0.0, Some(30));
        assert_eq!(pts.len(), 7);

        // All points lie on the vertical axis (x ≈ center_x) and are finite
        let cx = context.layout.center_px.x;
        for p in &pts {
            assert!(p.x.is_finite() && p.y.is_finite());
            assert!(approx(p.x, cx, 1e-7), "x={} vs cx={}", p.x, cx);
        }
    }

    #[test]
    fn sample_dec_parallel_filters_backside_for_gnomonic() {
        let context = make_context(|_| {});
        // dec=0, RA step 60 → 0,60,120,180,240,300; visible are 0,60,300 → 3
        let pts = super::sample_dec_parallel(&context, 0.0, Some(60));
        assert_eq!(pts.len(), 3);
    }

    #[test]
    fn split_segments_splits_on_large_jumps() {
        let pts = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 10.0, y: 0.0 },
            Point { x: 15.0, y: 0.0 },
            Point { x: 1000.0, y: 0.0 }, // big jump
            Point { x: 1005.0, y: 0.0 },
        ];
        let segs = super::split_segments(&pts, 100.0);
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].len(), 3);
        assert_eq!(segs[1].len(), 2);
    }
}
