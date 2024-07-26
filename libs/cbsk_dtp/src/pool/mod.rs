use std::sync::Arc;
use cbsk_mut_data::mut_data_vec::MutDataVec;
use crossbeam_deque::Worker;
use crate::job::Job;
use crate::pool::pool_config::PoolConfig;
use crate::task::Task;

pub mod pool_config;

/// dynamic thread pool
pub struct ThreadPool {
    /// thread config
    pub config: PoolConfig,
    /// worker
    worker: Worker<Task>,
    /// the job thread
    jobs: Arc<MutDataVec<Job>>,
}

/// custom method
impl ThreadPool {
    /// creates a FIFO worker queue thread pool<br />
    /// more see [Worker::new_fifo]
    pub fn new_fifo() -> Self {
        Self::new_fifo_config(PoolConfig::default())
    }

    /// creates a LIFO worker queue thread pool<br />
    /// more see [Worker::new_lifo]
    pub fn new_lifo() -> Self {
        Self::new_life_config(PoolConfig::default())
    }

    /// creates a FIFO worker queue thread pool withs config<br />
    /// more see [Worker::new_fifo]
    pub fn new_fifo_config(config: PoolConfig) -> Self {
        Self::new_woker(config, Worker::new_fifo())
    }
    /// creates a LIFO worker queue thread pool withs config<br />
    /// more see [Worker::new_lifo]
    pub fn new_life_config(config: PoolConfig) -> Self {
        Self::new_woker(config, Worker::new_lifo())
    }

    /// create by worker
    fn new_woker(config: PoolConfig, worker: Worker<Task>) -> Self {
        Self { config, worker, jobs: MutDataVec::with_capacity(1).into() }
    }

    /// add tasks to workers and wait for threads in the thread pool to steal them
    pub fn spawn(&self, f: impl FnOnce() + Send + 'static) {
        let a = self.worker.stealer();
        self.worker.push(Task::new(f))
    }

    /// start thread pool
    pub fn start(&self) {}
}
