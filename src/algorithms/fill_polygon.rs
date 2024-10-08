use std::collections::HashMap;
use std::usize;

use crate::geometry::Point;
use crate::geometry::{Line, Polygon};
use crate::nums::RoundToUsize;
use image::RgbaImage;
use itertools::Itertools;
use num_traits::{AsPrimitive, FromPrimitive, Num};
use palette::Srgba;
use palette::{blend::Blend, rgb::LinSrgba};

// the internal data structre for scan line algorithm
#[derive(Debug, Clone, Copy)]
struct ScanlineEdge {
    y_max: usize,
    x: f64,
    delta_x: f64,
    direction: i8,
}

impl ScanlineEdge {
    fn from_line<T>(line: Line<T>) -> Option<ScanlineEdge>
    where
        T: Copy + Num + PartialOrd + RoundToUsize + AsPrimitive<f64>,
    {
        if let Some(inv_slope) = line.inv_slope() {
            let y_max = line.y_max_point().y;
            let Point::<T> { x, y: y_min } = line.y_min_point();
            if y_max.ceil_to_usize() == y_min.ceil_to_usize() {
                // Almost horizontal
                return None;
            }
            Some(ScanlineEdge {
                y_max: y_max.floor_to_usize(),
                x: x.as_() + (y_min.ceil_to_self() - y_min).as_() * inv_slope,
                delta_x: inv_slope,
                direction: if line.start.y < line.end.y { 1 } else { -1 },
            })
        } else {
            // horizontal
            None
        }
    }

    fn shift_down(&mut self) {
        self.x = self.delta_x + self.x;
    }

    fn get_intersect(&self, rule: FillRule) -> (f64, i8) {
        match rule {
            FillRule::NonZero => (self.x, self.direction),
            FillRule::EvenOdd => (self.x, 1),
        }
    }
}

// New Edge Table
type Net = HashMap<usize, Vec<ScanlineEdge>>;

fn net_from_polygon<T>(poly: &Polygon<T>) -> Net
where
    T: Copy + Num + PartialOrd + RoundToUsize + AsPrimitive<f64>,
{
    let mut net = Net::new();
    for line in poly.edges() {
        if let Some(edge) = ScanlineEdge::from_line(line) {
            net.entry(line.y_min_point().y.ceil_to_usize())
                .or_insert_with(Vec::new)
                .push(edge)
        }
    }
    net
}

// Active Edge Table
type Aet = Vec<ScanlineEdge>;

#[derive(Copy, Clone)]
pub enum FillRule {
    NonZero,
    EvenOdd,
}

impl FillRule {
    fn check(&self, n: i32) -> bool {
        match self {
            Self::NonZero => n != 0,
            Self::EvenOdd => n % 2 != 0,
        }
    }
}

pub fn fill_polygon<T>(
    canvas: &mut RgbaImage,
    poly: &Polygon<T>,
    polygon_color: LinSrgba<f64>,
    rule: FillRule,
) where
    T: Copy + Num + PartialOrd + RoundToUsize + FromPrimitive + std::fmt::Debug + AsPrimitive<f64>,
{
    let height = canvas.height() as usize;

    // build NET
    let net = net_from_polygon(poly);
    //println!("net = {:?}", net);
    //println!();

    let mut aet = Aet::new();

    for row in 0..height {
        aet.iter_mut().for_each(|p| p.shift_down());
        aet.retain(|l| l.y_max > row);
        if let Some(new) = net.get(&row) {
            aet.extend(new.iter().cloned());
        }
        if aet.len() == 0 {
            // 快速跳过空行
            continue;
        }

        let internal_range = aet
            .iter()
            .map(|e| e.get_intersect(rule))
            // 按照交点排序
            .sorted_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .scan(0, |prefix_sum, point| {
                *prefix_sum += point.1 as i32; // 更新前缀和
                Some((point.0, rule.check(*prefix_sum))) // rule.check判断该点右侧是否是多边形内部
            })
            // non-zero rule 会产生连续的true和false
            // 连续的T/F除了第一个以外都无意义，删除
            .dedup_by(|p1, p2| p1.1 == p2.1)
            .map(|p| f64::ceil(p.0) as usize)
            .tuples::<(_, _)>();

        //println!();

        // 给多边形内部上色
        for (low_idx, high_idx) in internal_range {
            for col in low_idx..high_idx {
                let pixel = canvas.get_pixel_mut(col as u32, row as u32);
                let bg_color: LinSrgba<f64> = <&Srgba<u8>>::from(&pixel.0).into_linear();
                let blended = bg_color.multiply(polygon_color);
                println!(
                    "bg_color = {:?}, fg_color = {:?}, mixed_color = {:?}",
                    bg_color, polygon_color, blended
                );
                pixel.0 = Srgba::from_linear(blended).into();
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use approx::assert_relative_eq;
    use image::Rgba;

    fn assert_color_at(img: &RgbaImage, row: u32, col: u32, color: &LinSrgba<f64>) {
        let pixel = img.get_pixel(col, row);
        let linear_color: LinSrgba<f64> = <&Srgba<u8>>::from(&pixel.0).into_linear();
        assert_relative_eq!(linear_color, color);
    }

    fn empty_image() -> RgbaImage {
        RgbaImage::from_pixel(30, 20, Rgba([255, 255, 255, 255]))
    }

    #[test]
    fn test_scanline_full_float() {
        let poly = Polygon::from_vec(vec![0.0, 0.0, 8.0, 0.0, 8.0, 10.0, 0.0, 10.0]).unwrap();
        let black = LinSrgba::new(0f64, 0f64, 0f64, 1f64);
        let mut canvas = empty_image();
        fill_polygon(&mut canvas, &poly, black, FillRule::NonZero);
        println!("{:?}", canvas);
        assert_color_at(&canvas, 0, 0, &black);
        assert_color_at(&canvas, 9, 0, &black);
        assert_color_at(&canvas, 5, 5, &black);
        assert_color_at(&canvas, 0, 7, &black);
    }

    #[test]
    fn test_scanline_full_int() {
        let poly = Polygon::from_vec(vec![0, 0, 8, 0, 8, 10, 0, 10]).unwrap();
        let black = LinSrgba::new(0f64, 0f64, 0f64, 1f64);
        let mut canvas = empty_image();
        fill_polygon(&mut canvas, &poly, black, FillRule::NonZero);
        println!("{:?}", canvas);
        assert_color_at(&canvas, 0, 0, &black);
        assert_color_at(&canvas, 9, 0, &black);
        assert_color_at(&canvas, 5, 5, &black);
        assert_color_at(&canvas, 0, 7, &black);
    }

    #[test]
    fn test_scanline_triangle_float() {
        // lower triangle
        let poly = Polygon::from_vec(vec![0.0, 0.0, 8.0, 0.0, 8.0, 10.0]).unwrap();
        let black = LinSrgba::new(0f64, 0f64, 0f64, 1f64);
        let white = LinSrgba::new(1f64, 1f64, 1f64, 1f64);
        let mut canvas = empty_image();
        fill_polygon(&mut canvas, &poly, black, FillRule::NonZero);
        println!("{:?}", canvas);
        assert_color_at(&canvas, 0, 0, &black);
        assert_color_at(&canvas, 9, 0, &white);
        assert_color_at(&canvas, 5, 5, &black);
        assert_color_at(&canvas, 0, 7, &black);
    }
    #[test]
    fn test_scanline_triangle_int() {
        let poly = Polygon::from_vec(vec![0, 0, 8, 0, 8, 10]).unwrap();
        let black = LinSrgba::new(0f64, 0f64, 0f64, 1f64);
        let white = LinSrgba::new(1f64, 1f64, 1f64, 1f64);
        let mut canvas = empty_image();
        fill_polygon(&mut canvas, &poly, black, FillRule::NonZero);
        println!("{:?}", canvas);
        assert_color_at(&canvas, 0, 1, &black);
        assert_color_at(&canvas, 9, 0, &white);
        assert_color_at(&canvas, 5, 5, &black);
        assert_color_at(&canvas, 8, 7, &black);
    }

    #[test]
    fn test_scanline_rule_non_zero() {
        let poly = Polygon::from_vec(vec![0, 0, 20, 0, 3, 15, 13, 3, 8, 3, 18, 15]).unwrap();
        let black = LinSrgba::new(0f64, 0f64, 0f64, 1f64);
        let mut canvas = empty_image();
        fill_polygon(&mut canvas, &poly, black, FillRule::NonZero);
        for row in 0..15 {
            for col in 0..20 {
                print!("{} ", (canvas.get_pixel(col, row).0[0] > 0) as u8)
            }
            println!()
        }
        // no hole in self-intersect area
        assert_color_at(&canvas, 7, 10, &black);
    }

    #[test]
    fn test_scanline_rule_even_odd() {
        let poly = Polygon::from_vec(vec![0, 0, 20, 0, 3, 15, 13, 3, 8, 3, 18, 15]).unwrap();
        let black = LinSrgba::new(0f64, 0f64, 0f64, 1f64);
        let white = LinSrgba::new(1f64, 1f64, 1f64, 1f64);
        let mut canvas = empty_image();
        fill_polygon(&mut canvas, &poly, black, FillRule::EvenOdd);
        for row in 0..15 {
            for col in 0..20 {
                print!("{} ", (canvas.get_pixel(col, row).0[0] > 0) as u8);
            }
            println!()
        }
        // no hole in self-intersect area
        assert_color_at(&canvas, 7, 10, &white);
    }

    #[test]
    fn test_blending() {
        let square_left = Polygon::from_vec(vec![0, 0, 20, 0, 20, 10, 0, 10]).unwrap();
        let square_right = Polygon::from_vec(vec![10, 0, 30, 0, 30, 10, 0, 10]).unwrap();
        let white = LinSrgba::new(1f64, 1f64, 1f64, 1f64);
        let red = LinSrgba::new(1f64, 0f64, 0f64, 1f64);
        let green = LinSrgba::new(0f64, 1f64, 0f64, 1f64);
        let mut canvas = empty_image();
        fill_polygon(&mut canvas, &square_left, red, FillRule::EvenOdd);
        for row in 0..10 {
            for col in 0..30 {
                print!("{} ", (canvas.get_pixel(col, row).0[0] > 0) as u8);
            }
            println!()
        }
        assert_color_at(&canvas, 5, 15, &white.multiply(red));
        fill_polygon(&mut canvas, &square_right, green, FillRule::EvenOdd);
        assert_color_at(&canvas, 5, 15, &white.multiply(red).multiply(green));
    }
}
