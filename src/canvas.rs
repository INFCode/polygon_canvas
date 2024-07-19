use ndarray::Array2;
use palette::{
    blend::{Blend, PreAlpha},
    rgb::LinSrgb,
    LinSrgba,
};

#[derive(Debug)]
pub struct Canvas {
    canvas: Array2<PreAlpha<LinSrgb>>,
}

#[derive(Debug)]
pub struct CanvasSpec {
    pub width: usize,
    pub height: usize,
}

impl Canvas {
    pub fn from_spec(spec: &CanvasSpec) -> Self {
        Canvas {
            // fill with opaque black
            canvas: Array2::<PreAlpha<LinSrgb>>::from_elem(
                (spec.width, spec.height),
                PreAlpha::from(LinSrgba::new(0.0, 0.0, 0.0, 1.0)),
            ),
        }
    }

    pub fn get_spec(&self) -> CanvasSpec {
        let (width, height) = self.canvas.dim();
        CanvasSpec { width, height }
    }

    pub fn ref_as_image_buffer(&self) -> ImageBuffer {
        // TODO
    }
}
