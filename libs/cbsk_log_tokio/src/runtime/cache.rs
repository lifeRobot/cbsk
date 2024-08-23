use std::sync::LazyLock;
use cbsk_base::tokio;
use cbsk_base::tokio::sync::mpsc::UnboundedSender;
use cbsk_log::model::cbsk_record::CbskRecord;
use crate::receiver::Receiver;

/// global log cache
#[allow(non_upper_case_globals)]
pub static log_cache: LazyLock<LogCache> = LazyLock::new(LogCache::default);

/// log cache
pub struct LogCache {
    pub send: UnboundedSender<CbskRecord>,
    pub recv: Receiver,
}

/// support default
impl Default for LogCache {
    fn default() -> Self {
        let (send, recv) = tokio::sync::mpsc::unbounded_channel();
        Self { send, recv: Receiver::new(recv) }
    }
}

/// log cache is empty
pub fn is_empty() -> bool {
    let b = log_cache.recv.read().is_empty();
    b
}

/// push log to cache
pub fn push(record: CbskRecord) {
    if let Err(e) = log_cache.send.send(record) {
        eprintln!("send error: {e:?}");
    }
}