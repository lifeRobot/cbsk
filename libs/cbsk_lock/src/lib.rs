use std::sync::atomic::{AtomicBool, Ordering};

// TODO Test lock

#[derive(Default)]
pub struct TestLock {
    pub state: AtomicBool,
}

impl TestLock {
    pub fn lock(&self) {
        while let Err(_) = self.try_lock() {}
    }

    pub fn try_lock(&self) -> Result<(), String> {
        match self.state.compare_exchange(false, true, Ordering::Release, Ordering::Acquire) {
            Ok(_) => { Ok(()) }
            Err(_) => { Err("the lock has already been locked".into()) }
        }
    }
}

impl Drop for TestLock {
    fn drop(&mut self) {
        self.state.store(false, Ordering::Release);
    }
}
