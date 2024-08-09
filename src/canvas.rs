use image::Rgba32FImage;
use ndarray::Array2;
use palette::{
    cast::{ComponentsInto, IntoComponents},
    rgb::LinSrgba,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanvasSpec {
    pub width: usize,
    pub height: usize,
}

impl CanvasSpec {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Clone)]
pub struct Canvas {
    // This is the underlying vector holding the canvas content.
    // It should have the row major, or "c" order, memory layout.
    // Dimention order is [W, H, C].
    // Note that C = 4 here for pre-alpha color to speed up blending.
    buff: Vec<f32>,
    spec: CanvasSpec,
}

impl Canvas {
    pub fn from_spec(spec: &CanvasSpec) -> Self {
        // TODO: add check to spec to avoid panic when unwraping the Result from from_shape_vec
        Canvas {
            buff: vec![0f32; spec.width * spec.height * 4], // RGBA
            spec: *spec,
        }
    }

    pub fn get_spec(&self) -> CanvasSpec {
        return self.spec;
    }

    pub fn from_image(image: Rgba32FImage) -> Self {
        let width = image.width() as usize;
        let height = image.height() as usize;
        Canvas {
            buff: image.into_raw(),
            spec: CanvasSpec::new(width, height),
        }
    }

    pub fn from_array2(array: Array2<LinSrgba<f32>>) -> Self {
        let (height, width) = array.dim();
        let array = if !array.is_standard_layout() {
            // This will copy array using the standard layout
            array.as_standard_layout().into_owned()
        } else {
            array
        };
        let color_vec = array.into_raw_vec();
        Canvas {
            buff: color_vec.into_components(),
            spec: CanvasSpec { width, height },
        }
    }

    pub fn into_image(self) -> Rgba32FImage {
        // unwrap is safe because the shape always match the size of buffer
        Rgba32FImage::from_raw(self.spec.width as u32, self.spec.height as u32, self.buff).unwrap()
    }

    pub fn into_array2(self) -> Array2<LinSrgba> {
        let color_vec: Vec<LinSrgba> = self.buff.components_into();
        Array2::from_shape_vec((self.spec.height, self.spec.width), color_vec).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba32FImage;
    use ndarray::Array2;
    use palette::rgb::LinSrgba;

    #[test]
    fn test_canvas_creation() {
        let spec = CanvasSpec::new(10, 20);
        let canvas = Canvas::from_spec(&spec);
        assert_eq!(canvas.get_spec().width, 10);
        assert_eq!(canvas.get_spec().height, 20);
        assert_eq!(canvas.buff.len(), 10 * 20 * 4);
    }

    #[test]
    fn test_into_image() {
        let spec = CanvasSpec::new(10, 20);
        let canvas = Canvas::from_spec(&spec);
        let image = canvas.into_image();
        assert_eq!(image.width(), 10);
        assert_eq!(image.height(), 20);
        assert_eq!(image.as_raw().len(), 10 * 20 * 4);
    }

    #[test]
    fn test_into_array2() {
        let spec = CanvasSpec::new(10, 20);
        let canvas = Canvas::from_spec(&spec);
        let array = canvas.into_array2();
        assert_eq!(array.shape(), &[20, 10]);
    }

    #[test]
    fn test_from_image() {
        let width = 10u32;
        let height = 20u32;
        let image = Rgba32FImage::from_raw(width, height, vec![0.0; (width * height * 4) as usize])
            .unwrap();
        let canvas = Canvas::from_image(image);
        assert_eq!(canvas.get_spec().width, width as usize);
        assert_eq!(canvas.get_spec().height, height as usize);
        assert_eq!(canvas.buff.len(), (width * height * 4) as usize);
    }

    #[test]
    fn test_from_array2() {
        let width = 10;
        let height = 20;
        let array = Array2::<LinSrgba<f32>>::from_elem(
            (height, width),
            LinSrgba::new(0f32, 0f32, 0f32, 0f32),
        );
        let canvas = Canvas::from_array2(array);
        assert_eq!(canvas.get_spec().width, width);
        assert_eq!(canvas.get_spec().height, height);
        assert_eq!(canvas.buff.len(), width * height * 4);
    }

    #[test]
    fn test_conversion_round_trip() {
        let spec = CanvasSpec::new(10, 20);
        let canvas = Canvas::from_spec(&spec);

        // Convert to image and back
        let image = canvas.clone().into_image();
        let canvas_from_image = Canvas::from_image(image);
        assert_eq!(canvas.buff, canvas_from_image.buff);
        assert_eq!(canvas.get_spec(), canvas_from_image.get_spec());

        // Convert to array and back
        let array = canvas.clone().into_array2();
        let canvas_from_array = Canvas::from_array2(array);
        assert_eq!(canvas.buff, canvas_from_array.buff);
        assert_eq!(canvas.get_spec(), canvas_from_array.get_spec());
    }

    #[test]
    fn test_image_modification() {
        let spec = CanvasSpec::new(10, 20);
        let mut canvas = Canvas::from_spec(&spec);

        // Modify a specific pixel
        let index = (5 * spec.width + 3) * 4;
        canvas.buff[index] = 1.0;
        canvas.buff[index + 1] = 0.5;
        canvas.buff[index + 2] = 0.25;
        canvas.buff[index + 3] = 0.75;

        // Convert to image and check the pixel
        let image = canvas.into_image();
        let raw_image = image.as_raw();
        assert_eq!(raw_image[index], 1.0);
        assert_eq!(raw_image[index + 1], 0.5);
        assert_eq!(raw_image[index + 2], 0.25);
        assert_eq!(raw_image[index + 3], 0.75);
    }

    #[test]
    fn test_array_modification() {
        let spec = CanvasSpec::new(30, 20);
        let mut canvas = Canvas::from_spec(&spec);

        // Modify a specific pixel
        let index = (11 * spec.width + 2) * 4;
        canvas.buff[index] = 0.8;
        canvas.buff[index + 1] = 0.6;
        canvas.buff[index + 2] = 0.3;
        canvas.buff[index + 3] = 0.4;

        // Convert to array and check the pixel
        let array = canvas.into_array2();
        let pixel = array[(11, 2)];
        assert_eq!(pixel.color.red, 0.8);
        assert_eq!(pixel.color.green, 0.6);
        assert_eq!(pixel.color.blue, 0.3);
        assert_eq!(pixel.alpha, 0.4);
    }
}
