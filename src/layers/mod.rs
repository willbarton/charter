use svg::node::element::{Group, Text as TextEl};

use crate::context::ChartContext;

pub trait Layer {
    /// Produce an SVG group for this layer.
    fn render(&self, context: &ChartContext<'_>) -> Group;
}

pub fn group_with_class(class: &str) -> Group {
    let mut g = Group::new();
    g = g.set("class", class);
    g
}

pub fn text(class: &str, x: f64, y: f64, anchor: &str, content: &str) -> TextEl {
    TextEl::new(content)
        .set("class", class)
        .set("x", x)
        .set("y", y)
        .set("text-anchor", anchor)
}

pub mod constellations;
pub mod ecliptic;
pub mod frame;
pub mod grid;
pub mod labels;
pub mod objects;
pub mod stars;
pub mod zenith;

pub use constellations::ConstellationsLayer;
pub use ecliptic::EclipticLayer;
pub use frame::FrameLayer;
pub use grid::GridLayer;
pub use labels::LabelsLayer;
pub use objects::ObjectsLayer;
pub use stars::StarsLayer;
pub use zenith::ZenithLayer;
