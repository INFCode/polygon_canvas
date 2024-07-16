use super::Line;
use super::Point;

#[derive(Debug, PartialEq, Clone)]
pub struct Polygon<T: Copy> {
    pub vertices: Vec<Point<T>>,
}

impl<T> Polygon<T> where
    T: Copy
        + std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>
        + std::ops::Mul<Output = T>
        + PartialOrd
        + std::convert::From<f64>
{
}

impl<T: Copy> Polygon<T> {
    pub fn new() -> Self {
        Polygon {
            vertices: Vec::new(),
        }
    }

    pub fn from_vec(v: Vec<T>) -> Option<Self> {
        if v.len() % 2 != 0 {
            return None;
        }

        let mut poly = Self::new();
        for i in (0..v.len()).step_by(2) {
            poly.add_point(Point {
                x: v[i],
                y: v[i + 1],
            });
        }
        Some(poly)
    }

    pub fn add_point(&mut self, point: Point<T>) -> &mut Self {
        self.vertices.push(point);
        self
    }

    pub fn edges(&self) -> impl Iterator<Item = Line<T>> + '_ {
        return self
            .vertices
            .iter()
            .zip(self.vertices.iter().cycle().skip(1))
            .map(|(&x, &y)| Line { start: x, end: y });
    }
}

#[cfg(test)]
mod polygon_tests {
    use super::Polygon;
    use crate::geometry::Point;

    #[test]
    fn test_polygon_creation() {
        let polygon: Polygon<f64> = Polygon::new();
        assert_eq!(polygon.vertices.len(), 0);
    }

    #[test]
    fn test_add_point() {
        let mut polygon = Polygon::new();
        let p = Point::new(1.0, 2.0);
        polygon.add_point(p);
        assert_eq!(polygon.vertices.len(), 1);
        assert_eq!(polygon.vertices[0], p);
    }
}
