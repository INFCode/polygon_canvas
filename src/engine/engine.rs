use std::path::Path;

use crate::canvas::{Canvas, CanvasSpec};

pub struct Engine {
    canvas: Canvas,
    prev_score: f64,
}

impl Engine {
    pub fn new(spec: &CanvasSpec, image_path: &Path) -> Option<Self> {
        Some(Self {
            canvas: Canvas::from_spec(spec),
            prev_score: 0f64,
        })
    }
}
