use crate::config::ChartConfig;
use crate::context::{ChartContext, Datasets};
use crate::layers::{
    ConstellationsLayer, EclipticLayer, FrameLayer, GridLayer, LabelsLayer, Layer, ObjectsLayer,
    StarsLayer, ZenithLayer,
};
use std::fs;
use svg::node::element::{ClipPath, Definitions, Group, Rectangle, Style};
use svg::Document;

// Load the default css for embedding
const DEFAULT_CSS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/styles/chart.css"));

pub struct Chart<'a> {
    pub context: ChartContext<'a>,
    css_path: Option<String>,
}

impl<'a> Chart<'a> {
    pub fn new(data: Datasets<'a>, cfg: ChartConfig, css_path: Option<String>) -> Self {
        Self {
            context: ChartContext::new(data, cfg),
            css_path,
        }
    }

    fn load_css_text(&self) -> String {
        if let Some(path) = &self.css_path {
            if let Ok(text) = fs::read_to_string(path) {
                return text;
            }
        }
        // Embedded fallback
        DEFAULT_CSS.to_owned()
    }
    pub fn draw_document(&self) -> Document {
        let w = self.context.cfg.width;
        let h = self.context.cfg.height;
        let l = &self.context.layout;

        // Layer stack, back to front
        let clipped_layers: Vec<Box<dyn Layer>> = vec![
            Box::new(EclipticLayer::new()),
            Box::new(GridLayer::new()),
            Box::new(ConstellationsLayer::new()),
            Box::new(ObjectsLayer::new()),
            Box::new(StarsLayer::new()),
            Box::new(LabelsLayer::new()),
            Box::new(ZenithLayer::new()),
        ];
        let unclipped_layers: Vec<Box<dyn Layer>> = vec![Box::new(FrameLayer::new())];

        let mut doc = Document::new()
            .set("xmlns", "http://www.w3.org/2000/svg")
            .set("width", w)
            .set("height", h)
            .set("class", "chart");

        let css = self.load_css_text();
        if !css.is_empty() {
            doc = doc.add(Style::new(css));
        }

        let clip_rect = Rectangle::new()
            .set("x", l.plot_x)
            .set("y", l.plot_y)
            .set("width", l.plot_w)
            .set("height", l.plot_h);
        let clip = ClipPath::new().set("id", "clip-chart").add(clip_rect);
        let defs = Definitions::new().add(clip);
        doc = doc.add(defs);

        // Clipped layers that are inside the graticle borders
        let mut clipped = Group::new().set("clip-path", "url(#clip-chart)");
        for layer in clipped_layers {
            clipped = clipped.add(layer.render(&self.context));
        }
        doc = doc.add(clipped);

        // Unclipped layers outside the graticle borders
        for layer in unclipped_layers {
            doc = doc.add(layer.render(&self.context));
        }

        doc
    }

    pub fn to_file(&self, path: &str) -> std::io::Result<()> {
        let doc = self.draw_document();
        svg::save(path, &doc)
    }
}
