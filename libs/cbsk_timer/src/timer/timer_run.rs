use crate::timer::Timer;
use crate::timer::timer_state::TimerState;

/// timer tasks run logic
pub struct TimerRun {
    /// the last task run time
    pub last_time: i64,
    /// the task time state
    pub timer_state: TimerState,
    /// timer task
    pub timer: Box<dyn Timer>,
}

/// custom method
impl TimerRun {
    /// create loop task
    pub fn new(task: impl Timer) -> Self {
        Self {
            last_time: Self::now(),
            timer_state: TimerState::Ready,
            timer: Box::new(task),
        }
    }

    /// get now millis<br />
    /// more see [fastdate::DateTime::unix_timestamp_millis]
    pub fn now() -> i64 {
        fastdate::DateTime::now().unix_timestamp_millis()
    }

    /// the task is ready
    pub fn is_ready(&self) -> bool {
        if let TimerState::Running = self.timer_state {
            return false;
        }

        let interval = cbsk_base::match_some_return!(self.timer.interval(),true);
        let now = Self::now();
        let diff = now - self.last_time;
        // if time jumps back to the past, return ready true
        if diff < 0 {
            return true;
        }
        let diff = u128::try_from(diff).unwrap_or_default();
        diff >= interval.as_millis()
    }

    /// set state is running
    pub fn running(&mut self) {
        self.timer_state = TimerState::Running;
    }

    /// set state is ready and set last time is now
    pub fn ready(&mut self) {
        self.last_time = Self::now();
        self.timer_state = TimerState::Ready;
    }
}
