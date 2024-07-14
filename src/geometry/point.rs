#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T: Copy> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Point { x, y }
    }
}

#[cfg(test)]
mod point_tests {
    use super::Point;

    #[test]
    fn test_point_creation() {
        let p = Point::new(1.0, 2.0);
        assert_eq!(p.x, 1.0);
        assert_eq!(p.y, 2.0);
    }
}
