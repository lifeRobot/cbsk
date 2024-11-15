use std::sync::atomic::{AtomicUsize, Ordering};
use cbsk_base::log;
use cbsk_base::parking_lot::RwLock;
use crate::pool::pool_detail::PoolDetail;

pub mod pool_detail;

/// thread pool
pub struct Pool {
    /// thread pool
    pub thread_pool: RwLock<Vec<PoolDetail>>,
    /// number of thread pools, default is 100<br />
    /// each thread pool has ten threads, so the maximum number of threads is 10 * thread_pool_num
    pub thread_pool_num: AtomicUsize,
}

/// support default
impl Default for Pool {
    fn default() -> Self {
        let mut thread_pool = Vec::with_capacity(2);
        thread_pool.push(PoolDetail::build());
        Self {
            thread_pool: thread_pool.into(),
            thread_pool_num: AtomicUsize::new(100),
        }
    }
}

/// custom method
impl Pool {
    /// add once tasks to thread pool<br />
    /// will not detect if threads are idle<br />
    /// if immediate operation is required, please call [Self::is_idle] first to determine if the thread pool is idle
    pub fn spawn(&self, f: impl FnOnce() + Send + 'static) {
        let mut thread_pool = self.thread_pool.write();
        for pool in thread_pool.iter() {
            if pool.is_idle() {
                pool.spawn(f);
                return;
            }
        }

        // if not idle, build thread pool
        let pool =
            match PoolDetail::try_build() {
                Ok(pool) => { pool }
                Err(e) => {
                    log::error!("build thread fail:{e:?}");
                    return;
                }
            };
        pool.spawn(f);
        thread_pool.push(pool);
    }

    /// is there any idle thread in the thread pool
    pub fn is_idle(&self) -> bool {
        let thread_pool = self.thread_pool.read();
        if thread_pool.len() < self.thread_pool_num.load(Ordering::Acquire) {
            return true;
        }
        cbsk_base::match_some_return!(thread_pool.last(),false).is_idle()
    }
}
