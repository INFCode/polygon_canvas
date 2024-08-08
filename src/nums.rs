use num_traits::AsPrimitive;

pub trait Round {
    fn round_to_self(&self) -> Self;
    fn ceil_to_self(&self) -> Self;
    fn floor_to_self(&self) -> Self;
}

macro_rules! impl_round_to_self_float {
    ($($t:ty),*) => {
        $(
            impl Round for $t {
                fn round_to_self(&self) -> Self {
                    self.round()
                }

                fn ceil_to_self(&self) -> Self {
                    self.ceil()
                }

                fn floor_to_self(&self) -> Self {
                    self.floor()
                }
            }
        )*
    }
}

macro_rules! impl_round_to_self_int {
    ($($t:ty),*) => {
        $(
            impl Round for $t {
                fn round_to_self(&self) -> Self {
                    *self
                }

                fn ceil_to_self(&self) -> Self {
                    *self
                }

                fn floor_to_self(&self) -> Self {
                    *self
                }
            }
        )*
    }
}

impl_round_to_self_float!(f32, f64);
impl_round_to_self_int!(i32, i64, u32, u64);

pub trait RoundToUsize: Round {
    fn round_to_usize(&self) -> usize;
    fn ceil_to_usize(&self) -> usize;
    fn floor_to_usize(&self) -> usize;
}

impl<T> RoundToUsize for T
where
    T: AsPrimitive<usize> + Round,
{
    fn round_to_usize(&self) -> usize {
        self.round_to_self().as_()
    }
    fn ceil_to_usize(&self) -> usize {
        self.ceil_to_self().as_()
    }
    fn floor_to_usize(&self) -> usize {
        self.floor_to_self().as_()
    }
}

#[cfg(test)]
mod tests {
    use super::RoundToUsize;

    #[test]
    fn test_round_to_usize_f32() {
        let value: f32 = 5.7;
        assert_eq!(value.round_to_usize(), 6);
        assert_eq!(value.ceil_to_usize(), 6);
        assert_eq!(value.floor_to_usize(), 5);
    }

    #[test]
    fn test_round_to_usize_f64() {
        let value: f64 = 5.7;
        assert_eq!(value.round_to_usize(), 6);
        assert_eq!(value.ceil_to_usize(), 6);
        assert_eq!(value.floor_to_usize(), 5);
    }

    #[test]
    fn test_round_to_usize_i32() {
        let value: i32 = 42;
        assert_eq!(value.round_to_usize(), 42);
        assert_eq!(value.ceil_to_usize(), 42);
        assert_eq!(value.floor_to_usize(), 42);
    }

    #[test]
    fn test_round_to_usize_i64() {
        let value: i64 = 100;
        assert_eq!(value.round_to_usize(), 100);
        assert_eq!(value.ceil_to_usize(), 100);
        assert_eq!(value.floor_to_usize(), 100);
    }

    #[test]
    fn test_round_to_usize_u32() {
        let value: u32 = 42;
        assert_eq!(value.round_to_usize(), 42);
        assert_eq!(value.ceil_to_usize(), 42);
        assert_eq!(value.floor_to_usize(), 42);
    }

    #[test]
    fn test_round_to_usize_u64() {
        let value: u64 = 100;
        assert_eq!(value.round_to_usize(), 100);
        assert_eq!(value.ceil_to_usize(), 100);
        assert_eq!(value.floor_to_usize(), 100);
    }
}
