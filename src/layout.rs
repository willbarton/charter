use crate::config::ChartConfig;
use crate::types::Point;

#[derive(Debug, Clone, Copy)]
pub struct ChartLayout {
    pub plot_x: f64,
    pub plot_y: f64,
    pub plot_w: f64,
    pub plot_h: f64,
    pub center_px: Point,
    pub scale: f64,
    pub split_threshold: f64,
}

impl From<&ChartConfig> for ChartLayout {
    fn from(cfg: &ChartConfig) -> Self {
        let plot_x = cfg.margin.left as f64;
        let plot_y = cfg.margin.top as f64;
        let plot_w = (cfg.width - cfg.margin.left - cfg.margin.right) as f64;
        let plot_h = (cfg.height - cfg.margin.top - cfg.margin.bottom) as f64;
        let center_px = Point {
            x: plot_x + plot_w / 2.0,
            y: plot_y + plot_h / 2.0,
        };

        let half_fov_rad = (cfg.fov_deg / 2.0).to_radians();
        let rho_max = half_fov_rad.tan();
        let radius_px = plot_w.min(plot_h) / 2.0;
        let scale = radius_px / rho_max;

        let split_threshold = plot_w.min(plot_h) * 0.8;

        Self {
            plot_x,
            plot_y,
            plot_w,
            plot_h,
            center_px,
            scale,
            split_threshold,
        }
    }
}
