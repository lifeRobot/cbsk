use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
#[cfg(feature = "debug_mode")]
use cbsk_base::log;
use rayon::{ThreadPool, ThreadPoolBuilder};

/// thread pool detail
#[derive(Clone)]
pub struct PoolDetail {
    /// thread pool
    pub thread_pool: Arc<ThreadPool>,
    /// number of threads already in use
    pub used: Arc<AtomicU8>,
    /// max thread number
    pub max: u8,
}

/// custom method
impl PoolDetail {
    /// build thread pool<br />
    /// if build fail, will be panic, more see [Self::try_build]
    pub fn build() -> Self {
        Self::try_build().expect("build thread pool fail")
    }

    /// try build thread pool
    pub fn try_build() -> Result<Self, rayon::ThreadPoolBuildError> {
        Ok(Self {
            thread_pool: ThreadPoolBuilder::new().num_threads(10).build()?.into(),
            used: Arc::new(AtomicU8::new(0)),
            max: 10,
        })
    }

    /// add once tasks to thread pool<br />
    /// will not detect if threads are idle<br />
    /// if immediate operation is required, please call [Self::is_idle] first to determine if the thread pool is idle
    pub fn spawn(&self, f: impl FnOnce() + Send + 'static) {
        if self.used.fetch_add(1, Ordering::Release) == u8::MAX {
            self.used.store(u8::MAX, Ordering::Release)
        }
        #[cfg(feature = "debug_mode")] {
            let used = self.used.load(Ordering::Acquire);
            log::info!("run thread, used is {used}");
        }
        let pool = self.clone();
        self.thread_pool.spawn(move || {
            f();
            if pool.used.fetch_sub(1, Ordering::Release) == u8::MIN {
                pool.used.store(u8::MIN, Ordering::Release)
            }
            #[cfg(feature = "debug_mode")] {
                let used = pool.used.load(Ordering::Acquire);
                log::info!("thread release, used is {used}");
            }
        })
    }

    /// is there any idle thread in the thread pool
    pub fn is_idle(&self) -> bool {
        self.used.load(Ordering::Acquire) < self.max
    }
}
