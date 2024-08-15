use std::sync::LazyLock;
use std::time::SystemTime;
use cbsk_base::parking_lot::Mutex;

pub mod util;

/// global snowflake
pub static SNOWFLAKE: LazyLock<Mutex<SnowFlake>> = LazyLock::new(|| { Mutex::new(SnowFlake::default()) });

/// snowflake
pub struct SnowFlake {
    /// default is 1723593600000(2024-08-14T00:00:00.000Z)
    epoch: u128,
    /// snowflake worker id
    worker_id: u128,
    sequence: u128,
    last_timestamp: u128,
}

/// support default
impl Default for SnowFlake {
    fn default() -> Self {
        Self {
            epoch: 1_723_593_600_000,
            worker_id: util::mac_worker_u128(),
            sequence: 0,
            last_timestamp: 0,
        }
    }
}

/// get set method etc.
impl SnowFlake {
    /// max worker id
    pub const MAX_WORKER_ID: u16 = 1023;

    /// set epoch
    pub fn set_epoch(mut self, epoch: u128) -> Self {
        self.epoch = epoch;
        self
    }

    /// set worker id
    pub fn set_worker_id(mut self, worker_id: u16) -> Self {
        if worker_id > Self::MAX_WORKER_ID { return self; }
        self.worker_id = u128::from(worker_id) << 12;
        self
    }

    /// get epch
    pub fn get_epoch(&self) -> u128 {
        self.epoch
    }

    /// get worker id
    pub fn get_worker_id(&self) -> u16 {
        u16::try_from(self.worker_id >> 12).unwrap_or_default()
    }
}

/// business logic
impl SnowFlake {
    /// get next snowflake id
    pub fn next(&mut self) -> u128 {
        let time = self.get_timestamp();
        if time <= self.last_timestamp {
            self.sequence += 1;
            return (self.last_timestamp - self.epoch) << 22 | self.worker_id | self.sequence;
        }

        // time > last_time
        self.last_timestamp = time;
        self.sequence = 0;
        (self.last_timestamp - self.epoch) << 22 | self.worker_id
    }

    /// get next snowflake id by u64
    pub fn next_u64(&mut self) -> u64 {
        u64::try_from(self.next()).unwrap_or_default()
    }

    /// get next snowflake id by i64
    pub fn next_i64(&mut self) -> i64 {
        i64::try_from(self.next()).unwrap_or_default()
    }

    /// get timestamp
    fn get_timestamp(&self) -> u128 {
        let time = SystemTime::now();

        match time.duration_since(std::time::UNIX_EPOCH) {
            Ok(time) => { time.as_millis() }
            Err(_) => { self.last_timestamp }
        }
    }
}

/// get next snowflake id
pub fn next() -> u128 {
    SNOWFLAKE.lock().next()
}

/// get next snowflake id by u64
pub fn next_u64() -> u64 {
    SNOWFLAKE.lock().next_u64()
}

/// get next snowflake id by i64
pub fn next_i64() -> i64 {
    SNOWFLAKE.lock().next_i64()
}
