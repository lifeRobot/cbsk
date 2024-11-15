use std::sync::atomic::{AtomicI64, AtomicU8, Ordering};
use crate::timer::Timer;
use crate::timer::timer_state::TimerState;

/// timer tasks run logic
pub struct TimerRun {
    /// the last task run time
    pub last_time: AtomicI64,
    /// the task time state
    pub timer_state: AtomicU8,
    /// timer task
    pub timer: Box<dyn Timer>,
}

/// suppoet sync
unsafe impl Sync for TimerRun {}
/// suppoet send
unsafe impl Send for TimerRun {}

/// custom method
impl TimerRun {
    /// create loop task
    pub fn new(task: impl Timer) -> Self {
        Self {
            last_time: AtomicI64::new(Self::now()),
            timer_state: AtomicU8::new(TimerState::READY),
            timer: Box::new(task),
        }
    }

    /// get now millis<br />
    /// more see [fastdate::DateTime::unix_timestamp_millis]
    pub fn now() -> i64 {
        cbsk_base::fastdate::DateTime::now().unix_timestamp_millis()
    }

    /// the task is ready
    pub fn is_ready(&self) -> bool {
        if TimerState::RUNNING == self.timer_state.load(Ordering::Acquire) {
            return false;
        }

        let interval = cbsk_base::match_some_return!(self.timer.interval(),true);
        let now = Self::now();
        let diff = now - self.last_time.load(Ordering::Acquire);
        // if time jumps back to the past, return ready true
        if diff < 0 {
            return true;
        }
        let diff = u128::try_from(diff).unwrap_or_default();
        diff >= interval.as_millis()
    }

    /// set state is running
    pub fn running(&self) {
        self.timer_state.store(TimerState::RUNNING, Ordering::Release);
    }

    /// set state is ready and set last time is now
    pub fn ready(&self) {
        self.last_time.store(Self::now(), Ordering::Release);
        self.timer_state.store(TimerState::READY, Ordering::Release);
    }
}
