use std::sync::Arc;
use std::thread;
use std::time::Duration;
#[cfg(feature = "debug_mode")]
use cbsk_base::log;
use cbsk_base::once_cell::sync::Lazy;
use cbsk_mut_data::mut_data_obj::MutDataObj;
use cbsk_mut_data::mut_data_vec::MutDataVec;
use crate::pool::Pool;
use crate::timer::timer_run::TimerRun;

/// global runtime
#[allow(non_upper_case_globals)]
pub static runtime: Lazy<Runtime> = Lazy::new(Runtime::default);

/// global runtime
pub struct Runtime {
    /// once tasks
    pub(crate) once: Arc<MutDataVec<Box<dyn FnOnce() + Send>>>,
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
        while !self.once.is_empty() {
            if !self.pool.is_idle() {
                return;
            }

            let task = self.once.remove(0);
            #[cfg(feature = "debug_mode")]
            log::info!("run once");
            self.pool.spawn(task);
        }
    }

    /// run timer tasks
    fn run_timer(&self) {
        for (i, t) in self.timer.iter().enumerate() {
            // if task is end, remove task and return
            if t.timer.ended() {
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
            // not idle thread pool, next
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
