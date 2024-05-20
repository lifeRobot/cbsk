use std::time::Duration;

pub(crate) mod timer_state;
pub(crate) mod timer_run;
pub mod simple_timer;
pub(crate) mod once;

/// timer tasks
pub trait Timer: 'static {
    /// timer name
    fn name(&self) -> &str;

    /// timer run logic
    fn run(&self);

    /// ren before, default return true<br />
    /// please execute the non blocking logic that ends immediately<br />
    /// do nothing when return false<br />
    /// execute [Self::run] in the thread pool when return true
    fn run_before(&self) -> bool { true }

    /// the timer is ended<br />
    /// default is false
    fn ended(&self) -> bool { false }

    /// task loop interval<br />
    /// if return None, will run continuously
    fn interval(&self) -> Option<Duration> {
        None
    }

    /// start timer
    fn start(self) where Self: Sized {
        super::push_timer(self);
        super::run();
    }
}