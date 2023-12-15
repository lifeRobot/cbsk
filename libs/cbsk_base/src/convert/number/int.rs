/// build from num fn
macro_rules! build_from_num_fn {
    ($f:ident,$ty:ty) => {
        #[doc = concat!("from ",stringify!($ty))]
        fn $f(num: $ty) -> Self {
            Self::from_i128(i128::from(num))
        }
    };
}

/// build to num fn
macro_rules! build_to_num_fn {
    ($f:ident,$ty:ty) => {
        #[doc = concat!("to ",stringify!($ty))]
        fn $f(&self) -> $ty {
            <$ty>::from(self.to_i8())
        }
    };
}

/// from int to data
pub trait FromInt: Sized {
    build_from_num_fn!(from_i8,i8);
    build_from_num_fn!(from_i16,i16);
    build_from_num_fn!(from_i32,i32);
    build_from_num_fn!(from_i64,i64);

    /// from isize
    fn from_isize(num: isize) -> Self {
        Self::from_i128(i128::try_from(num).unwrap_or_default())
    }

    /// from i128
    fn from_i128(num: i128) -> Self;
}

/// data to int
pub trait ToInt {
    /// to i8
    fn to_i8(&self) -> i8;
    build_to_num_fn!(to_i16,i16);
    build_to_num_fn!(to_i32,i32);
    build_to_num_fn!(to_i64,i64);
    build_to_num_fn!(to_i128,i128);
    build_to_num_fn!(to_isize,isize);
}
