/// run time loop
pub enum RunTimeLoop {
    /// run once
    Once,
    /// run timer
    // Timer(usize),
    Timer,
}

/// support default
impl Default for RunTimeLoop {
    fn default() -> Self {
        Self::Once
    }
}
