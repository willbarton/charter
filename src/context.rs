use crate::types::{CelestialObject, Constellation};
use crate::{config::ChartConfig, layout::ChartLayout};

pub struct Datasets<'a> {
    pub stars: &'a [CelestialObject],
    pub objects: &'a [CelestialObject],
    pub constellations: &'a [Constellation],
}

pub struct ChartContext<'a> {
    pub data: Datasets<'a>,
    pub cfg: ChartConfig,
    pub layout: ChartLayout,
}

impl<'a> ChartContext<'a> {
    pub fn new(data: Datasets<'a>, cfg: ChartConfig) -> Self {
        let layout = ChartLayout::from(&cfg);
        Self { data, cfg, layout }
    }

    /// Adaptive step based on FOV
    pub fn adaptive_step_deg(&self) -> u32 {
        let fov_deg = self.cfg.fov_deg;
        let target = if fov_deg <= 30.0 {
            240.0
        } else if fov_deg <= 60.0 {
            180.0
        } else {
            120.0
        };
        let step = (fov_deg / target).clamp(0.5, 4.0).round() as u32;
        step.max(1)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::make_context;

    #[test]
    fn adaptive_step_deg_matches_buckets_and_clamping() {
        // ≤30° bucket → step rounds to 1°
        assert_eq!(
            make_context(|cfg| cfg.fov_deg = 10.0).adaptive_step_deg(),
            1
        );
        assert_eq!(
            make_context(|cfg| cfg.fov_deg = 30.0).adaptive_step_deg(),
            1
        );

        // 30–60° bucket → still 1°
        assert_eq!(
            make_context(|cfg| cfg.fov_deg = 45.0).adaptive_step_deg(),
            1
        );
        assert_eq!(
            make_context(|cfg| cfg.fov_deg = 60.0).adaptive_step_deg(),
            1
        );

        // >60° bucket
        assert_eq!(
            make_context(|cfg| cfg.fov_deg = 120.0).adaptive_step_deg(),
            1
        ); // 120/120=1
        assert_eq!(
            make_context(|cfg| cfg.fov_deg = 180.0).adaptive_step_deg(),
            2
        ); // 1.5→2
        assert_eq!(
            make_context(|cfg| cfg.fov_deg = 300.0).adaptive_step_deg(),
            3
        ); // 2.5→3

        // Very large FOV clamps at 4°
        assert_eq!(
            make_context(|cfg| cfg.fov_deg = 1000.0).adaptive_step_deg(),
            4
        );
    }

    #[test]
    fn adaptive_step_is_monotonic_non_decreasing_with_fov() {
        let fovs = [
            5.0, 10.0, 20.0, 30.0, 45.0, 60.0, 90.0, 120.0, 180.0, 300.0, 1000.0,
        ];
        let mut prev = 0;
        for &f in &fovs {
            let step = make_context(|cfg| cfg.fov_deg = f).adaptive_step_deg();
            assert!(
                step >= prev,
                "step should not decrease as FOV grows (fov={f}, step={step}, prev={prev})"
            );
            assert!((1..=4).contains(&step)); // clamped range
            prev = step;
        }
    }
}
