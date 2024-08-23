use std::ops::Deref;
use cbsk_base::parking_lot::{RawRwLock};
use cbsk_base::parking_lot::lock_api::RwLockReadGuard;
use cbsk_base::tokio::sync::mpsc::UnboundedReceiver;
use cbsk_log::model::cbsk_record::CbskRecord;

/// parking_log rwlock read guard
pub struct ReadGuard<'a> {
    guard: RwLockReadGuard<'a, RawRwLock, UnboundedReceiver<CbskRecord>>,
}

/// custom method
impl<'a> ReadGuard<'a> {
    /// create read guard
    pub fn new(guard: RwLockReadGuard<'a, RawRwLock, UnboundedReceiver<CbskRecord>>) -> Self {
        Self { guard }
    }
}

/// support deref
impl Deref for ReadGuard<'_> {
    type Target = UnboundedReceiver<CbskRecord>;

    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}
