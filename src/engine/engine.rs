use std::path::Path;

use num_traits::Num;

use crate::canvas::{Canvas, CanvasSpec};

pub struct Engine<T> {
    canvas: Canvas<T>,
    prev_score: f64,
}

impl<T> Engine<T>
where
    T: Num + Clone,
{
    pub fn new(spec: &CanvasSpec, image_path: &Path) -> Option<Self> {
        Some(Self {
            canvas: Canvas::<T>::from_spec(spec),
            prev_score: 0f64,
        })
    }
}
