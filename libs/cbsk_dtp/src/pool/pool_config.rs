use std::thread;

/// thread pool config
pub struct PoolConfig {
    /// max thread size
    pub max: usize,

}

/// support default
impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max: thread::available_parallelism().map(|num| { num.get() }).unwrap_or_default()
        }
    }
}
