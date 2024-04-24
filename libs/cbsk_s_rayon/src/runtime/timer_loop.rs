use std::time::Duration;

/// task loop state
pub enum TimerLoop {
    /// just run once
    Once,
    /// run by interval
    Interval(Duration),
}