/// task timer state
pub struct TimerState {}

/// timer state enum
impl TimerState {
    /// the task is ready
    pub const READY: u8 = 0;
    /// the task is running
    pub const RUNNING: u8 = 1;
}


