use std::sync::{Arc, LazyLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
#[cfg(feature = "debug_mode")]
use cbsk_base::log;
use cbsk_base::parking_lot::RwLock;
use crate::pool::Pool;
use crate::timer::once::Once;
use crate::timer::timer_run::TimerRun;

/// global runtime
#[allow(non_upper_case_globals)]
pub static runtime: LazyLock<Runtime> = LazyLock::new(Runtime::default);

/// global runtime
pub struct Runtime {
    /// once tasks
    pub(crate) once: RwLock<Vec<Once>>,
    /// timer tasks
    pub(crate) timer: RwLock<Vec<Arc<TimerRun>>>,
    /// thread pool
    pub(crate) pool: Pool,
    /// is global runtime running
    running: AtomicBool,
}

/// support sync
unsafe impl Sync for Runtime {}

/// support default
impl Default for Runtime {
    fn default() -> Self {
        Self {
            once: Vec::with_capacity(2).into(),
            timer: Vec::with_capacity(2).into(),
            pool: Pool::default(),
            running: AtomicBool::default(),
        }
    }
}

/// custom method
impl Runtime {
    /// start the global runtime
    pub fn start(&self) {
        if self.running.load(Ordering::Acquire) { return; }

        self.run();
    }

    /// global runtime logic
    fn run(&self) {
        self.running.store(true, Ordering::Release);

        self.pool.spawn(|| {
            loop {
                runtime.run_once();
                runtime.run_timer();

                thread::sleep(Duration::from_millis(10));
            }
        })
    }
}

/// custom method
impl Runtime {
    /// run once tasks
    fn run_once(&self) {
        let mut once = self.once.write();
        if once.is_empty() {
            return;
        }

        if !self.pool.is_idle() {
            return;
        }

        let once = once.remove(0);
        #[cfg(feature = "debug_mode")]
        log::info!("remove once {}",once.name);
        self.pool.spawn(|| {
            let _name = once.name;
            let task = once.once;
            #[cfg(feature = "debug_mode")]
            log::info!("{_name} run once");
            task();
            #[cfg(feature = "debug_mode")]
            log::info!("{_name} once release");
        });
    }

    /// run timer tasks
    fn run_timer(&self) {
        let mut timer = self.timer.write();
        for (i, t) in timer.iter().enumerate() {
            // if task is end, remove task and return
            if t.timer.ended() {
                #[cfg(feature = "debug_mode")]
                log::info!("remove timer {}", t.timer.name());
                timer.remove(i);
                return;
            }

            // can't run before, next
            if !t.timer.run_before() {
                continue;
            }
            // if not ready, next
            if !t.is_ready() {
                continue;
            }
            // not idle thread pool, return and wait pool idle
            if !self.pool.is_idle() {
                continue;
            }

            // if task is can't run
            t.running();
            let t = t.clone();
            self.pool.spawn(move || {
                #[cfg(feature = "debug_mode")]
                log::info!("{} run timer", t.timer.name());

                t.timer.run();
                t.ready();

                #[cfg(feature = "debug_mode")]
                log::info!("{} timer release", t.timer.name());
            })
        }
    }
}
