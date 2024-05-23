use std::sync::Arc;
use std::thread;
use std::time::Duration;
#[cfg(feature = "debug_mode")]
use cbsk_base::log;
use cbsk_base::once_cell::sync::Lazy;
use cbsk_mut_data::mut_data_obj::MutDataObj;
use cbsk_mut_data::mut_data_vec::MutDataVec;
use crate::pool::Pool;
use crate::timer::once::Once;
use crate::timer::timer_run::TimerRun;

// mod runtime_loop;

/// global runtime
#[allow(non_upper_case_globals)]
pub static runtime: Lazy<Runtime> = Lazy::new(Runtime::default);

/// global runtime
pub struct Runtime {
    /// once tasks
    pub(crate) once: Arc<MutDataVec<Once>>,
    /// timer tasks
    pub(crate) timer: Arc<MutDataVec<Arc<MutDataObj<TimerRun>>>>,
    /// thread pool
    pub(crate) pool: Arc<Pool>,
    /// is global runtime running
    running: Arc<MutDataObj<bool>>,
}

/// support default
impl Default for Runtime {
    fn default() -> Self {
        Self {
            once: MutDataVec::with_capacity(2).into(),
            timer: MutDataVec::with_capacity(2).into(),
            pool: Pool::default().into(),
            running: Arc::new(MutDataObj::default()),
        }
    }
}

/// custom method
impl Runtime {
    /// start the global runtime
    pub fn start(&self) {
        if **self.running { return; }

        self.run();
    }

    /// global runtime logic
    fn run(&self) {
        self.running.set_true();

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
        if self.once.is_empty() {
            return;
        }

        if !self.pool.is_idle() {
            return;
        }

        let once = self.once.remove(0);
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
        for (i, t) in self.timer.iter().enumerate() {
            // if task is end, remove task and return
            if t.timer.ended() {
                #[cfg(feature = "debug_mode")]
                log::info!("remove timer {}", t.timer.name());
                self.timer.remove(i);
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
            t.as_mut().running();
            let t = t.clone();
            self.pool.spawn(move || {
                #[cfg(feature = "debug_mode")]
                log::info!("{} run timer", t.timer.name());

                t.timer.run();
                t.as_mut().ready();

                #[cfg(feature = "debug_mode")]
                log::info!("{} timer release", t.timer.name());
            })
        }
    }
}
