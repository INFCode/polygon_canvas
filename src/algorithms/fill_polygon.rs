use std::collections::HashMap;
use std::usize;

use crate::geometry::{Line, Polygon};
use crate::nums::RoundToUsize;
use crate::{canvas::CanvasSpec, geometry::Point};
use ndarray::Array2;
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
        if let Some(new) = net.get(&row) {
            aet.extend(new.iter().cloned());
        }
        aet.retain(|l| l.y_max >= row);
        if aet.len() == 0 {
            continue;
        }
        //println!("aet = {:?}", aet);

        let mut points = aet
            .iter()
            .map(|e| e.get_intersect(rule))
            .collect::<Vec<(f64, i8)>>();
        points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        //println!("points = {:?}", points);
        //println!();

        let mut idx = 0;
        let mut track = 0;
        for col in 0..spec.width {
            while idx < points.len() && col as f64 >= points[idx].0 {
                track += points[idx].1 as i32;
                println!("{} {}", col, track);
                idx += 1;
            }
            mask[[row, col]] = rule.check(track);
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
