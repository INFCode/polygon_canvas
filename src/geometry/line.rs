use num_traits::Num;

use super::Point;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Line<T> {
    pub start: Point<T>,
    pub end: Point<T>,
}

impl<T> Line<T> {
    pub fn new(start: Point<T>, end: Point<T>) -> Self {
        Line { start, end }
    }
}

impl<T> Line<T>
where
    T: Copy + Num,
{
    pub fn inv_slope(&self) -> Option<T> {
        if self.start.y == self.end.y {
            None // Vertical line
        } else {
            Some((self.end.x - self.start.x) / (self.end.y - self.start.y))
        }
    }
}

impl<T> Line<T>
where
    T: Copy + PartialOrd,
{
    pub fn y_min_point(&self) -> Point<T> {
        if self.start.y <= self.end.y {
            self.start
        } else {
            self.end
        }
    }

    pub fn y_max_point(&self) -> Point<T> {
        if self.start.y >= self.end.y {
            self.start
        } else {
            self.end
        }
    }
}

#[cfg(test)]
mod line_tests {
    use super::{Line, Point};

    #[test]
    fn test_line_creation() {
        let p1 = Point::new(1.0, 2.0);
        let p2 = Point::new(4.0, 6.0);
        let line = Line::new(p1, p2);
        assert_eq!(line.start, p1);
        assert_eq!(line.end, p2);
    }

    #[test]
    fn test_line_slope() {
        let p1 = Point::new(1.0, 2.0);
        let p2 = Point::new(4.0, 3.0);
        let line = Line::new(p1, p2);
        assert_eq!(line.inv_slope(), Some(3.0));
    }

    #[test]
    fn test_vertical_line_slope() {
        let p1 = Point::new(1.0, 2.0);
        let p2 = Point::new(6.0, 2.0);
        let line = Line::new(p1, p2);
        assert_eq!(line.inv_slope(), None);
    }

    #[test]
    fn test_lower_endpoint() {
        let p1 = Point::new(1.0, 2.0);
        let p2 = Point::new(4.0, 6.0);
        let line = Line::new(p1, p2);
        assert_eq!(line.y_min_point(), p1);
    }

    #[test]
    fn test_higher_endpoint() {
        let p1 = Point::new(1.0, 2.0);
        let p2 = Point::new(4.0, 6.0);
        let line = Line::new(p1, p2);
        assert_eq!(line.y_max_point(), p2);
    }
}
