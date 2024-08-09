use std::collections::HashMap;
use std::usize;

use crate::geometry::{Line, Polygon};
use crate::nums::RoundToUsize;
use crate::{canvas::CanvasSpec, geometry::Point};
use itertools::Itertools;
use ndarray::{s, Array2};
use num_traits::{AsPrimitive, FromPrimitive, Num};

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

fn net_from_polygon<T>(poly: Polygon<T>) -> Net
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

pub fn fill_polygon<T>(poly: Polygon<T>, spec: CanvasSpec, rule: FillRule) -> Array2<bool>
where
    T: Copy + Num + PartialOrd + RoundToUsize + FromPrimitive + std::fmt::Debug + AsPrimitive<f64>,
{
    let mut mask = Array2::<bool>::from_elem((spec.height, spec.width), false);

    // build NET
    let net = net_from_polygon(poly);
    //println!("net = {:?}", net);
    //println!();

    let mut aet = Aet::new();

    for row in 0..spec.height {
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
            .sorted_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .scan(0, |prefix_sum, point| {
                *prefix_sum += point.1 as i32; // 更新前缀和
                Some((point.0, rule.check(*prefix_sum))) // rule.check判断该点右侧是否是多边形内部
            })
            // non-zero rule 会产生连续的true和false
            // 连续的T/F除了第一个以外都无意义，删除
            .dedup_by(|p1, p2| p1.1 == p2.1)
            // 创建一个滑动窗口
            .tuple_windows()
            //
            .filter_map(|(current, next)| {
                if current.1 {
                    // 如果 current 是 true
                    Some((current.0, next.0)) // 返回 (current.0, next.0)
                } else {
                    None
                }
            });
        //println!("points = {:?}", points);
        //println!();
        for (low, high) in internal_range {
            let low_idx = f64::ceil(low) as usize;
            let high_idx = f64::ceil(high) as usize;
            mask.slice_mut(s![row, low_idx..high_idx])
                .map_inplace(|b| *b = true)
        }
    }

    return mask;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_scanline_full_float() {
        let poly = Polygon::from_vec(vec![0.0, 0.0, 8.0, 0.0, 8.0, 10.0, 0.0, 10.0]).unwrap();
        let spec = CanvasSpec {
            width: 8,
            height: 10,
        };
        let result = fill_polygon(poly, spec, FillRule::NonZero);
        println!("{:?}", result);
        assert!(result[[0, 0]]);
        assert!(result[[9, 0]]);
        assert!(result[[5, 5]]);
        assert!(result[[0, 7]]);
    }

    #[test]
    fn test_scanline_full_int() {
        let poly = Polygon::from_vec(vec![0, 0, 8, 0, 8, 10, 0, 10]).unwrap();
        let spec = CanvasSpec {
            width: 8,
            height: 10,
        };
        let result = fill_polygon(poly, spec, FillRule::NonZero);
        println!("{:?}", result);
        assert!(result[[0, 0]]);
        assert!(result[[9, 0]]);
        assert!(result[[5, 5]]);
        assert!(result[[0, 7]]);
    }

    #[test]
    fn test_scanline_triangle_float() {
        // lower triangle
        let poly = Polygon::from_vec(vec![0.0, 0.0, 8.0, 0.0, 8.0, 10.0]).unwrap();
        let spec = CanvasSpec {
            width: 8,
            height: 10,
        };
        let result = fill_polygon(poly, spec, FillRule::NonZero);
        println!("{:?}", result);
        assert!(result[[0, 1]]);
        assert!(!result[[9, 0]]);
        assert!(result[[5, 5]]);
        assert!(result[[8, 7]]);
    }

    #[test]
    fn test_scanline_triangle_int() {
        let poly = Polygon::from_vec(vec![0, 0, 8, 0, 8, 10]).unwrap();
        let spec = CanvasSpec {
            width: 8,
            height: 10,
        };
        let result = fill_polygon(poly, spec, FillRule::NonZero);
        println!("{:?}", result);
        assert!(result[[0, 1]]);
        assert!(!result[[9, 0]]);
        assert!(result[[5, 5]]);
        assert!(result[[8, 7]]);
    }

    #[test]
    fn test_scanline_rule_non_zero() {
        let poly = Polygon::from_vec(vec![0, 0, 20, 0, 3, 15, 13, 3, 8, 3, 18, 15]).unwrap();
        let spec = CanvasSpec {
            width: 20,
            height: 15,
        };
        let result = fill_polygon(poly, spec, FillRule::NonZero);
        for row in 0..15 {
            for col in 0..20 {
                print!("{} ", if result[[row, col]] { 1 } else { 0 });
            }
            println!()
        }
        // no hole in self-intersect area
        assert!(result[[7, 10]]);
    }

    #[test]
    fn test_scanline_rule_even_odd() {
        let poly = Polygon::from_vec(vec![0, 0, 20, 0, 3, 15, 13, 3, 8, 3, 18, 15]).unwrap();
        let spec = CanvasSpec {
            width: 20,
            height: 15,
        };
        let result = fill_polygon(poly, spec, FillRule::EvenOdd);
        for row in 0..15 {
            for col in 0..20 {
                print!("{} ", if result[[row, col]] { 1 } else { 0 });
            }
            println!()
        }
        // hole in self-intersect area
        assert!(!result[[7, 10]]);
    }
}
