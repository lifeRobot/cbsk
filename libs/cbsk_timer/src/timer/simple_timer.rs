use std::time::Duration;
use cbsk_mut_data::mut_data_obj::MutDataObj;

/// simple timer tasks
pub struct SimpleTimer {
    /// timer name
    pub name: String,
    /// whether to end the task
    pub(crate) end: MutDataObj<bool>,
    /// task
    pub task: Box<dyn Fn(&Self)>,
    /// task loop interval
    pub interval: Duration,
}

/// support timer
impl super::Timer for SimpleTimer {
    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn run(&self) {
        (self.task)(self);
    }

    fn ended(&self) -> bool {
        *self.end
    }

    fn interval(&self) -> Option<Duration> {
        Some(self.interval)
    }
}

/// custom method
impl SimpleTimer {
    /// create simple loop task
    pub fn new(name: impl Into<String>, interval: Duration, task: impl Fn(&Self) + 'static) -> Self {
        Self {
            name: name.into(),
            end: MutDataObj::default(),
            task: Box::new(task),
            interval,
        }
    }

    /// notification timer task needs to end
    pub fn task_end(&self) {
        self.end.set_true();
    }
}
