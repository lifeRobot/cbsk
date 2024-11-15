use std::sync::atomic::Ordering;
use std::time::Duration;
#[cfg(feature = "debug_mode")]
use cbsk_base::log;
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
    push_once_with_name("default", task);
}

/// push once task with once name<br />
/// please do not use dead loops in tasks
pub fn push_once_with_name(name: impl Into<String>, task: impl FnOnce() + Send + 'static) {
    let once = Once::new(name, task);
    let mut once_write = runtime::runtime.once.write();
    #[cfg(feature = "debug_mode")]{
        log::info!("push once {}", once.name);
        log::info!("before once num {}",once_write.len());
    }
    once_write.push(once);
    #[cfg(feature = "debug_mode")]{
        log::info!("after once num {}",once_write.len());
        if let Some(last) = once_write.last() {
            log::info!("last once is {}",last.name);
        }
    }
}

/// push custom timer<br />
/// more see souce code [runtime::Runtime::run_timer]<br />
/// please do not use dead loops in [Timer::run_before] and [Timer::run]
pub fn push_timer(timer: impl Timer) {
    #[cfg(feature = "debug_mode")]
    log::info!("push timer {}",timer.name());
    runtime::runtime.timer.write().push(TimerRun::new(timer).into())
}

/// push interval task<br />
/// please do not use dead loops in tasks
pub fn push_task(name: impl Into<String>, interval: Duration, task: impl Fn(&SimpleTimer) + 'static) {
    push_timer(SimpleTimer::new(name, interval, task))
}

/// set number of thread pools
pub fn set_thread_pool_num(thread_pool_num: usize) {
    runtime::runtime.pool.thread_pool_num.store(thread_pool_num, Ordering::Release)
}

/// get tasks num
pub fn tasks_num() -> usize {
    let once_len = runtime::runtime.once.read().len();
    let timer_len = runtime::runtime.timer.read().len();
    #[cfg(feature = "debug_mode")] {
        log::info!("timer len is {once_len}");
        log::info!("once len is {timer_len}");
    }
    once_len + timer_len
}
