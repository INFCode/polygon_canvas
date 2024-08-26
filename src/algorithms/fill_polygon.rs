use std::collections::HashMap;
use std::usize;

use crate::geometry::Point;
use crate::geometry::{Line, Polygon};
use crate::nums::RoundToUsize;
use itertools::Itertools;
use ndarray::{s, Array2};
use num_traits::{AsPrimitive, FromPrimitive, Num};
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
    canvas: &mut Array2<LinSrgba>,
    poly: &Polygon<T>,
    color: LinSrgba,
    rule: FillRule,
) where
    T: Copy + Num + PartialOrd + RoundToUsize + FromPrimitive + std::fmt::Debug + AsPrimitive<f64>,
{
    let (height, width) = canvas.dim();

    // build NET
    let net = net_from_polygon(poly);
    //println!("net = {:?}", net);
    //println!();

    let mut aet = Aet::new();

    for row in 0..height {
        aet.iter_mut().for_each(|p| p.shift_down());
        // 下面的if let和retain顺序不能换，extend进去的还可能立刻被移除
        if let Some(new) = net.get(&row) {
            aet.extend(new.iter().cloned());
        }
        aet.retain(|l| l.y_max >= row);
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
        for (low_idx, high_idx) in internal_range {
            canvas
                .slice_mut(s![row, low_idx..high_idx])
                .map_inplace(|c| *c = c.burn(color))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_scanline_full_float() {
        let poly = Polygon::from_vec(vec![0.0, 0.0, 8.0, 0.0, 8.0, 10.0, 0.0, 10.0]).unwrap();
        let black = LinSrgba::new(0f32, 0f32, 0f32, 1f32);
        let white = LinSrgba::new(1f32, 1f32, 1f32, 1f32);
        let mut canvas = Array2::from_elem((10, 8), black);
        fill_polygon(&mut canvas, &poly, white, FillRule::NonZero);
        println!("{:?}", canvas);
        assert_eq!(canvas[[0, 0]], white);
        assert_eq!(canvas[[9, 0]], white);
        assert_eq!(canvas[[5, 5]], white);
        assert_eq!(canvas[[0, 7]], white);
    }

    #[test]
    fn test_scanline_full_int() {
        let poly = Polygon::from_vec(vec![0, 0, 8, 0, 8, 10, 0, 10]).unwrap();
        let black = LinSrgba::new(0f32, 0f32, 0f32, 1f32);
        let white = LinSrgba::new(1f32, 1f32, 1f32, 1f32);
        let mut canvas = Array2::from_elem((10, 8), black);
        fill_polygon(&mut canvas, &poly, white, FillRule::NonZero);
        println!("{:?}", canvas);
        assert_eq!(canvas[[0, 0]], white);
        assert_eq!(canvas[[9, 0]], white);
        assert_eq!(canvas[[5, 5]], white);
        assert_eq!(canvas[[0, 7]], white);
    }

    #[test]
    fn test_scanline_triangle_float() {
        // lower triangle
        let poly = Polygon::from_vec(vec![0.0, 0.0, 8.0, 0.0, 8.0, 10.0]).unwrap();
        let black = LinSrgba::new(0f32, 0f32, 0f32, 1f32);
        let white = LinSrgba::new(1f32, 1f32, 1f32, 1f32);
        let mut canvas = Array2::from_elem((10, 8), black);
        fill_polygon(&mut canvas, &poly, white, FillRule::NonZero);
        println!("{:?}", canvas);
        assert_eq!(canvas[[0, 1]], white);
        assert_eq!(canvas[[9, 0]], black);
        assert_eq!(canvas[[5, 5]], white);
        assert_eq!(canvas[[8, 7]], white);
    }

    #[test]
    fn test_scanline_triangle_int() {
        let poly = Polygon::from_vec(vec![0, 0, 8, 0, 8, 10]).unwrap();
        let black = LinSrgba::new(0f32, 0f32, 0f32, 1f32);
        let white = LinSrgba::new(1f32, 1f32, 1f32, 1f32);
        let mut canvas = Array2::from_elem((10, 8), black);
        fill_polygon(&mut canvas, &poly, white, FillRule::NonZero);
        println!("{:?}", canvas);
        assert_eq!(canvas[[0, 1]], white);
        assert_eq!(canvas[[9, 0]], black);
        assert_eq!(canvas[[5, 5]], white);
        assert_eq!(canvas[[8, 7]], white);
    }

    #[test]
    fn test_scanline_rule_non_zero() {
        let poly = Polygon::from_vec(vec![0, 0, 20, 0, 3, 15, 13, 3, 8, 3, 18, 15]).unwrap();
        let black = LinSrgba::new(0f32, 0f32, 0f32, 1f32);
        let white = LinSrgba::new(1f32, 1f32, 1f32, 1f32);
        let mut canvas = Array2::from_elem((15, 20), black);
        fill_polygon(&mut canvas, &poly, white, FillRule::NonZero);
        for row in 0..15 {
            for col in 0..20 {
                print!("{} ", if canvas[[row, col]].red > 0.0 { 1 } else { 0 });
            }
            println!()
        }
        // no hole in self-intersect area
        assert_eq!(canvas[[7, 10]], white);
    }

    #[test]
    fn test_scanline_rule_even_odd() {
        let poly = Polygon::from_vec(vec![0, 0, 20, 0, 3, 15, 13, 3, 8, 3, 18, 15]).unwrap();
        let black = LinSrgba::new(0f32, 0f32, 0f32, 1f32);
        let white = LinSrgba::new(1f32, 1f32, 1f32, 1f32);
        let mut canvas = Array2::from_elem((15, 20), black);
        fill_polygon(&mut canvas, &poly, white, FillRule::EvenOdd);
        for row in 0..15 {
            for col in 0..20 {
                print!("{} ", if canvas[[row, col]].red > 0.0 { 1 } else { 0 });
            }
            println!()
        }
        // no hole in self-intersect area
        assert_eq!(canvas[[7, 10]], black);
    }

    #[test]
    fn test_blending() {
        let square_left = Polygon::from_vec(vec![0, 0, 20, 0, 20, 10, 0, 10]).unwrap();
        let square_right = Polygon::from_vec(vec![10, 0, 30, 0, 30, 10, 0, 10]).unwrap();
        let black = LinSrgba::new(0f32, 0f32, 0f32, 1f32);
        let red = LinSrgba::new(1f32, 0f32, 0f32, 1f32);
        let green = LinSrgba::new(0f32, 1f32, 0f32, 1f32);
        let mut canvas = Array2::from_elem((10, 30), black);
        fill_polygon(&mut canvas, &square_left, red, FillRule::EvenOdd);
        for row in 0..10 {
            for col in 0..30 {
                print!("{} ", if canvas[[row, col]].red > 0.0 { 1 } else { 0 });
            }
            println!()
        }
        assert_eq!(canvas[[5, 15]], black.burn(red));
        fill_polygon(&mut canvas, &square_right, green, FillRule::EvenOdd);
        assert_eq!(canvas[[5, 15]], black.burn(red).burn(green));
    }
}
