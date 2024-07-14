use std::clone::Clone;

use ndarray::Array2;
use num_traits::Num;

#[derive(Debug)]
pub struct Canvas<T> {
    canvas: Array2<T>,
}

#[derive(Debug)]
pub struct CanvasSpec {
    pub x: usize,
    pub y: usize,
}

impl<T> Canvas<T>
where
    T: Num + Clone,
{
    pub fn from_spec(spec: &CanvasSpec) -> Self {
        Canvas {
            canvas: Array2::<T>::from_elem((spec.x, spec.y), T::zero()),
        }
    }
}
