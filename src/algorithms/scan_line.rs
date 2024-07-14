use std::collections::HashMap;
use std::usize;

use crate::geometry::{Line, Polygon};
use crate::nums::RoundToUsize;
use crate::{canvas::CanvasSpec, geometry::Point};
use ndarray::Array2;
use num_traits::{FromPrimitive, Num};

// the internal data structre for scan line algorithm
#[derive(Clone, Copy)]
struct ScanlineEdge<T> {
    y_max: usize,
    x: T,
    delta_x: T,
    is_upwards: bool,
}

impl<T> ScanlineEdge<T>
where
    T: Copy + Num + PartialOrd + RoundToUsize,
{
    fn from_line(line: Line<T>) -> Option<ScanlineEdge<T>> {
        if let Some(inv_slope) = line.inv_slope() {
            let y_max = line.y_max_point().y;
            let Point::<T> { x, y: y_min } = line.y_min_point();
            if y_max.ceil_to_usize() == y_min.ceil_to_usize() {
                // Almost horizontal
                return None;
            }
            Some(ScanlineEdge {
                y_max: y_max.floor_to_usize(),
                x: x + (y_min.ceil_to_self() - y_min) * inv_slope,
                delta_x: inv_slope,
                is_upwards: line.start.y < line.end.y,
            })
        } else {
            // horizontal
            None
        }
    }

    fn shift_down(&mut self, rule: FillRule) -> (T, bool) {
        self.x = self.delta_x + self.x;
        match rule {
            FillRule::NonZero => (self.x, self.is_upwards),
            FillRule::EvenOdd => (self.x, true),
        }
    }
}

// New Edge Table
type Net<T> = HashMap<usize, Vec<ScanlineEdge<T>>>;

fn net_from_polygon<T>(poly: Polygon<T>) -> Net<T>
where
    T: Copy + Num + PartialOrd + RoundToUsize,
{
    let mut net = Net::new();
    for line in poly.edges() {
        if let Some(edge) = ScanlineEdge::from_line(line) {
            net.entry(line.y_min_point().y.ceil_to_usize())
                .and_modify(|vec| vec.push(edge))
                .or_insert_with(|| vec![edge]);
        }
    }
    net
}

// Active Edge Table
type Aet<T> = Vec<ScanlineEdge<T>>;

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

pub fn polygon_interior<T>(poly: Polygon<T>, spec: CanvasSpec, rule: FillRule) -> Array2<bool>
where
    T: Copy + Num + PartialOrd + RoundToUsize + FromPrimitive,
{
    let mut mask = Array2::<bool>::from_elem((spec.y, spec.x), false);

    // build NET
    let net = net_from_polygon(poly);

    let mut aet = Aet::<T>::new();

    for row in 0..spec.y {
        if let Some(new) = net.get(&row) {
            aet.extend(new);
        }
        aet.retain(|l| l.y_max <= row);

        let mut points = aet
            .iter_mut()
            .map(|e| e.shift_down(rule))
            .collect::<Vec<(T, bool)>>();
        points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let mut idx = 0;
        let mut track = 0;
        for col in 0..spec.x {
            let col_t = T::from_usize(col).unwrap();
            while idx < points.len() && col_t >= points[idx].0 {
                track += points[idx].1 as i32 * 2 - 1; // true mapped to 1 and false mapped to -1
                idx += 1;
            }
            mask[[row, col]] = rule.check(track);
        }
    }

    return mask;
}
