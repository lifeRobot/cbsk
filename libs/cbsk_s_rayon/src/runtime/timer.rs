use std::time::Duration;
#[cfg(feature = "debug_mode")]
use cbsk_base::log;
use cbsk_mut_data::mut_data_obj::MutDataObj;
use crate::runtime::timer_state::TimerState;

/// timed tasks
pub struct Timer {
    /// timer name
    pub name: String,
    /// whether to end the task
    pub(crate) end: MutDataObj<bool>,
    /// the last task run time
    last_time: MutDataObj<i64>,
    /// the task time state
    timer_state: MutDataObj<TimerState>,
    /// task
    task: Box<dyn Fn(&Self) + Sync + Send>,
    /// task loop interval
    interval: Duration,
}

impl Timer {
    /// create loop task
    pub fn new(name: impl Into<String>, interval: Duration, task: impl Fn(&Self) + Sync + Send + 'static) -> Self {
        Self {
            name: name.into(),
            end: MutDataObj::new(false),
            last_time: MutDataObj::new(Self::now()),
            timer_state: MutDataObj::new(TimerState::Ready),
            task: Box::new(task),
            interval,
        }
    }

    /// task is end
    pub fn task_end(&self) {
        self.end.set_true();
    }

    /// get now millis<br />
    /// more see [fastdate::DateTime::unix_timestamp_millis]
    pub fn now() -> i64 {
        fastdate::DateTime::now().unix_timestamp_millis()
    }

    /// the task is ready
    pub fn ready(&self) -> bool {
        if let TimerState::Running = self.timer_state.as_ref() {
            return false;
        }

        let now = Self::now();
        let diff = now - *self.last_time;
        // if time jumps back to the past, return ready true
        if diff < 0 {
            return true;
        }
        let diff = u128::try_from(diff).unwrap_or_default();
        diff >= self.interval.as_millis()
    }

    /// run task
    pub(crate) fn run(&self) {
        #[cfg(feature = "debug_mode")]
        log::info!("timer run name is {}",self.name);
        self.timer_state.set(TimerState::Running);
        (self.task)(self);

        self.last_time.set(Self::now());
        self.timer_state.set(TimerState::Ready);
    }
}
