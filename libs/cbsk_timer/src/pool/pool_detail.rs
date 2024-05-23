use std::sync::Arc;
#[cfg(feature = "debug_mode")]
use cbsk_base::log;
use cbsk_mut_data::mut_data_obj::MutDataObj;
use rayon::{ThreadPool, ThreadPoolBuilder};

/// thread pool detail
#[derive(Clone)]
pub struct PoolDetail {
    /// thread pool
    pub thread_pool: Arc<ThreadPool>,
    /// number of threads already in use
    pub used: Arc<MutDataObj<u8>>,
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
            used: Arc::new(MutDataObj::default()),
            max: 10,
        })
    }

    /// add once tasks to thread pool<br />
    /// will not detect if threads are idle<br />
    /// if immediate operation is required, please call [Self::is_idle] first to determine if the thread pool is idle
    pub fn spawn(&self, f: impl FnOnce() + Send + 'static) {
        self.used.set(self.used.saturating_add(1));
        #[cfg(feature = "debug_mode")]
        log::info!("run thread, used is {}",self.used);
        let pool = self.clone();
        self.thread_pool.spawn(move || {
            f();
            pool.used.set(pool.used.saturating_sub(1));
            #[cfg(feature = "debug_mode")]
            log::info!("thread release, used is {}",pool.used);
        })
    }

    /// is there any idle thread in the thread pool
    pub fn is_idle(&self) -> bool {
        **self.used < self.max
    }
}
