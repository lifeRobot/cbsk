/// log size<br />
/// used to split log files
pub enum LogSize {
    B(usize),
    KB(usize),
    MB(usize),
    GB(usize),
    TB(usize),
    EB(usize),
}

/// custom method
impl LogSize {
    /// get log size
    pub fn len(&self) -> usize {
        match self {
            Self::B(b) => *b,
            Self::KB(kb) => kb * 1024,
            Self::MB(mb) => mb * 1024 * 1024,
            Self::GB(gb) => gb * 1024 * 1024 * 1024,
            Self::TB(tb) => tb * 1024 * 1024 * 1024 * 1024,
            Self::EB(eb) => eb * 1024 * 1024 * 1024 * 1024 * 1024,
        }
    }
}
