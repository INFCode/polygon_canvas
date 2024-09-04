use image::{Rgba, RgbaImage};
use palette::rgb::LinSrgba;
use polygon_canvas::{
    algorithms::fill_polygon::{fill_polygon, FillRule},
    geometry::Polygon,
};

fn main() {
    let blk_size = 50;
    let num_row = 3;
    let num_col = 4;

    let mut canvas = RgbaImage::from_pixel(
        num_col * blk_size,
        num_row * blk_size,
        Rgba([255, 255, 255, 255]),
    );
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
                0.2 + row_blk as f64 * 0.35,
                0.9 - col_blk as f64 * 0.25,
                0.7f64,
                1f64,
            );

            fill_polygon(&mut canvas, &square, color, FillRule::NonZero);
        }
    }

    let _ = canvas
        .save_with_format("./render_polygon.png", image::ImageFormat::Png)
        .or_else(|err| -> Result<(), ()> {
            println!("{}", err);
            Ok(())
        });
}
