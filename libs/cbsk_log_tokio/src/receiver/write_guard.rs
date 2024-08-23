use std::ops::{Deref, DerefMut};
use cbsk_base::parking_lot::lock_api::RwLockWriteGuard;
use cbsk_base::parking_lot::RawRwLock;
use cbsk_base::tokio::sync::mpsc::UnboundedReceiver;
use cbsk_log::model::cbsk_record::CbskRecord;

/// parking_log rwlock write guard
pub struct WriteGuard<'a> {
    guard: RwLockWriteGuard<'a, RawRwLock, UnboundedReceiver<CbskRecord>>,
}

/// support send
unsafe impl Send for WriteGuard<'_> {}

/// custom method
impl<'a> WriteGuard<'a> {
    /// create write guard
    pub fn new(guard: RwLockWriteGuard<'a, RawRwLock, UnboundedReceiver<CbskRecord>>) -> Self {
        Self { guard }
    }
}

/// support deref
impl Deref for WriteGuard<'_> {
    type Target = UnboundedReceiver<CbskRecord>;

    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}

/// support deref mut
impl DerefMut for WriteGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.deref_mut()
    }
}
