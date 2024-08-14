use std::sync::atomic::AtomicU64;

pub mod util;

/// snowflake
pub struct SnowFlake {
    /// default is 1723593600000(2024-08-14T00:00:00.000Z)
    epoch: u64,
    /// snowfalke worker id
    worker_id: u64,
    sequence: AtomicU64,
    last_timestamp: AtomicU64,
}

/// support default
impl Default for SnowFlake {
    fn default() -> Self {
        Self {
            epoch: 1_723_593_600_000,
            worker_id: util::mac_worker_u64(),
            sequence: AtomicU64::new(0),
            last_timestamp: AtomicU64::new(0),
        }
    }
}

/// custom method
impl SnowFlake {
    /// max worker id
    pub const MAX_WORKER_ID: u16 = 1023;
}
