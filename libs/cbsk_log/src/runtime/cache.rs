use std::sync::LazyLock;
use crossbeam::channel::{Receiver, Sender};
use crate::model::cbsk_record::CbskRecord;

/// global log cache
#[allow(non_upper_case_globals)]
pub static log_cache: LazyLock<LogCache> = LazyLock::new(LogCache::default);

pub struct LogCache {
    pub send: Sender<CbskRecord>,
    pub recv: Receiver<CbskRecord>,
}

/// support default
impl Default for LogCache {
    fn default() -> Self {
        let (send, recv) = crossbeam::channel::unbounded();
        Self { send, recv }
    }
}

/// log cache is empty
pub fn is_empty() -> bool {
    log_cache.send.is_empty() || log_cache.recv.is_empty()
}

/// push log to cache
pub fn push(record: CbskRecord) {
    if let Err(e) = log_cache.send.send(record) {
        eprintln!("send error: {e:?}");
    }
}
