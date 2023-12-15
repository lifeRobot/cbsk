/// build from num fn
macro_rules! build_from_num_fn {
    ($f:ident,$ty:ty) => {
        #[doc = concat!("from ",stringify!($ty))]
        fn $f(num: $ty) -> Self {
            Self::from_u128(u128::from(num))
        }
    };
}

/// build to num fn
macro_rules! build_to_num_fn {
    ($f:ident,$ty:ty) => {
        #[doc = concat!("to ",stringify!($ty))]
        fn $f(&self) -> $ty {
            <$ty>::from(self.to_u8())
        }
    };
}

/// from uint to data
pub trait FromUInt: Sized {
    build_from_num_fn!(from_u8,u8);
    build_from_num_fn!(from_u16,u16);
    build_from_num_fn!(from_u32,u32);
    build_from_num_fn!(from_u64,u64);

    /// from usize
    fn from_usize(num: usize) -> Self {
        Self::from_u128(u128::try_from(num).unwrap_or_default())
    }

    /// from u128
    fn from_u128(num: u128) -> Self;
}

/// data to uint
pub trait ToUInt {
    /// to u8
    fn to_u8(&self) -> u8;
    build_to_num_fn!(to_u16,u16);
    build_to_num_fn!(to_u32,u32);
    build_to_num_fn!(to_u64,u64);
    build_to_num_fn!(to_u128,u128);
    build_to_num_fn!(to_usize,usize);
}
