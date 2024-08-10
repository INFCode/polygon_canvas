use std::result;

use image::DynamicImage;
use palette::rgb::LinSrgba;
use polygon_canvas::{
    algorithms::fill_polygon::{fill_polygon, FillRule},
    canvas::Canvas,
    geometry::Polygon,
};

fn main() {
    let blk_size = 50;
    let canvas = Canvas::from_wh(4 * blk_size, 3 * blk_size);

    let mut canvas_arr = canvas.into_array2();
    for row_blk in 0..3 {
        for col_blk in 0..4 {
            let row_offset = row_blk * blk_size;
            let col_offset = col_blk * blk_size;
            let square = Polygon::from_vec(vec![
                (0 + col_offset) as f32,
                (0 + row_offset) as f32,
                (blk_size + col_offset) as f32,
                (0 + row_offset) as f32,
                (blk_size + col_offset) as f32,
                (blk_size + row_offset) as f32,
                (0 + col_offset) as f32,
                (blk_size + row_offset) as f32,
            ])
            .unwrap();
            //println!("{:?}", square);
            let color = LinSrgba::new(
                0.2 + row_blk as f32 * 0.35,
                0.9 - col_blk as f32 * 0.25,
                0.7,
                1f32,
            );

            fill_polygon(&mut canvas_arr, &square, color, FillRule::NonZero);
        }
    }

    let image = Canvas::from_array2(canvas_arr).into_image();
    let _ = DynamicImage::ImageRgba32F(image)
        .to_rgba8()
        .save_with_format("./render_polygon.png", image::ImageFormat::Png)
        .or_else(|err| -> Result<(), ()> {
            println!("{}", err);
            Ok(())
        });
}
