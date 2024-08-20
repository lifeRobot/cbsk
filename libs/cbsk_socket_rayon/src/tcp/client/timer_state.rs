/// cbsk tcp timer state
pub enum TimerState {
    /// run conn
    Conn,
    /// run read data
    Read,
}

/// support default
impl Default for TimerState {
    fn default() -> Self {
        Self::Conn
    }
}
