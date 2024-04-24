use std::time::Duration;
use cbsk_mut_data::mut_data_obj::MutDataObj;
use crate::runtime::timer_loop::TimerLoop;
use crate::runtime::timer_state::TimerState;

/// timed tasks
pub struct Timer {
    /// tasks loop interval
    timer_loop: TimerLoop,
    /// whether to end the task
    pub(crate) end: MutDataObj<bool>,
    /// the last task run time
    last_time: MutDataObj<i64>,
    /// the task time state
    timer_state: MutDataObj<TimerState>,
    /// task
    task: Box<dyn Fn(&Self) + Sync + Send>,
}

impl Timer {
    /// one time task
    pub fn once(task: impl Fn(&Self) + Sync + Send + 'static) -> Self {
        Self {
            timer_loop: TimerLoop::Once,
            end: MutDataObj::new(false),
            last_time: MutDataObj::new(Self::now()),
            timer_state: MutDataObj::new(TimerState::Ready),
            task: Box::new(task),
        }
    }

    /// create loop task
    pub fn new(interval: Duration, task: impl Fn(&Self) + Sync + Send + 'static) -> Self {
        Self {
            timer_loop: TimerLoop::Interval(interval),
            end: MutDataObj::new(false),
            last_time: MutDataObj::new(Self::now()),
            timer_state: MutDataObj::new(TimerState::Ready),
            task: Box::new(task),
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

        let interval =
            match &self.timer_loop {
                TimerLoop::Once => { return true; }
                TimerLoop::Interval(interval) => { interval }
            };

        let now = Self::now();
        let diff = u128::try_from(now - *self.last_time).unwrap_or_default();
        diff >= interval.as_millis()
    }

    /// run task
    pub(crate) fn run(&self) {
        self.timer_state.set(TimerState::Running);
        (self.task)(self);

        if let TimerLoop::Once = &self.timer_loop {
            self.task_end();
            return;
        }

        self.last_time.set(Self::now());
        self.timer_state.set(TimerState::Ready);
    }
}
