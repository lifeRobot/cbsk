/// from float to data
pub trait FromFloat: Sized {
    /// from f32
    fn from_f32(num: f32) -> Self {
        Self::from_f64(f64::from(num))
    }

    /// from f64
    fn from_f64(num: f64) -> Self;
}

/// data to float
pub trait ToFloat {
    /// to f32
    fn to_f32(&self) -> f32;

    // to f64
    fn to_f64(&self) -> f64 {
        f64::from(self.to_f32())
    }
}
