use cbsk_base::parking_lot::RwLock;
use cbsk_base::tokio::sync::mpsc::UnboundedReceiver;
use cbsk_log::model::cbsk_record::CbskRecord;
use crate::receiver::read_guard::ReadGuard;
use crate::receiver::write_guard::WriteGuard;

pub mod read_guard;
pub mod write_guard;

/// rwlock recv
pub struct Receiver {
    /// recv
    recv: RwLock<UnboundedReceiver<CbskRecord>>,
}

/// custom method
impl Receiver {
    /// create receiver
    pub fn new(recv: UnboundedReceiver<CbskRecord>) -> Self {
        Self { recv: RwLock::new(recv) }
    }

    /// get read lock
    pub fn read(&self) -> ReadGuard<'_> {
        ReadGuard::new(self.recv.read())
    }

    /// get write lock
    pub fn write(&self) -> WriteGuard<'_> {
        WriteGuard::new(self.recv.write())
    }
}
