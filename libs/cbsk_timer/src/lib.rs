use std::time::Duration;
#[cfg(feature = "debug_mode")]
use cbsk_base::log;
use cbsk_mut_data::mut_data_obj::MutDataObj;
use crate::timer::once::Once;
use crate::timer::simple_timer::SimpleTimer;
use crate::timer::Timer;
use crate::timer::timer_run::TimerRun;

pub(crate) mod runtime;
pub(crate) mod pool;
pub mod timer;

/// start the global runtime
pub fn run() {
    runtime::runtime.start()
}

/// push once task<br />
/// please do not use dead loops in tasks
pub fn push_once(task: impl FnOnce() + Send + 'static) {
    runtime::runtime.once.push(Once::once(task));
}

/// push once task with once name<br />
/// please do not use dead loops in tasks
pub fn push_once_with_name(name: impl Into<String>, task: impl FnOnce() + Send + 'static) {
    runtime::runtime.once.push(Once::new(name, task));
}

/// push custom timer<br />
/// more see souce code [runtime::Runtime::run_timer]<br />
/// please do not use dead loops in [Timer::run_before] and [Timer::run]
pub fn push_timer(timer: impl Timer) {
    #[cfg(feature = "debug_mode")]
    log::info!("push timer {}",timer.name());
    runtime::runtime.timer.push(MutDataObj::new(TimerRun::new(timer)).into())
}

/// push interval task<br />
/// please do not use dead loops in tasks
pub fn push_task(name: impl Into<String>, interval: Duration, task: impl Fn(&SimpleTimer) + 'static) {
    push_timer(SimpleTimer::new(name, interval, task))
}

/// set number of thread pools
pub fn set_thread_pool_num(thread_pool_num: usize) {
    runtime::runtime.pool.thread_pool_num.set(thread_pool_num)
}

/// get tasks num
pub fn tasks_num() -> usize {
    #[cfg(feature = "debug_mode")] {
        log::info!("timer len is {}",runtime::runtime.timer.len());
        log::info!("once len is {}",runtime::runtime.once.len());
    }
    runtime::runtime.once.len() + runtime::runtime.timer.len()
}
