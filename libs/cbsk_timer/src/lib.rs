pub(crate) mod runtime;
pub(crate) mod pool;
pub mod timer;

/// start the global runtime
pub fn run() {
    runtime::runtime.start()
}

/// push once task<br />
/// please do not use dead loops in tasks
pub fn push_once(task: impl FnOnce() + Send + 'static) {
    runtime::runtime.once.push(Box::new(task));
}
