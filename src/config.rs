use crate::types::{EQPoint, Projection};

#[derive(Debug, Clone, Copy)]
pub struct Margin {
    pub top: u32,
    pub bottom: u32,
    pub left: u32,
    pub right: u32,
}
impl Margin {
    pub fn uniform(px: u32) -> Self {
        Self {
            top: px,
            bottom: px,
            left: px,
            right: px,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChartConfig {
    pub center: EQPoint,
    pub position_angle_deg: f64,
    pub projection: Projection,
    pub fov_deg: f64,
    pub width: u32,
    pub height: u32,
    pub margin: Margin,
    pub step_ra_deg: u32,
    pub step_dec_deg: u32,
    pub limit_star_mag: f64,
    pub limit_object_mag: f64,
    pub object_scale: f64,
}
impl Default for ChartConfig {
    fn default() -> Self {
        Self {
            center: EQPoint {
                ra_deg: 0.0,
                dec_deg: 0.0,
            },
            position_angle_deg: 0.0,
            projection: Projection::Gnomonic,
            fov_deg: 60.0,
            width: 800,
            height: 800,
            margin: Margin::uniform(40),
            step_ra_deg: 15,
            step_dec_deg: 10,
            limit_star_mag: 10.0,
            limit_object_mag: 11.0,
            object_scale: 1.0,
        }
    }
}
