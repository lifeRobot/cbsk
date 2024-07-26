/// the task
pub struct Task {
    /// task
    task: Box<dyn FnOnce() + Send>,
}

/// custom method
impl Task {
    /// create new task
    pub fn new(f: impl FnOnce() + Send + 'static) -> Self {
        Self { task: Box::new(f) }
    }

    /// run task
    pub fn run(self) {
        (self.task)();
    }
}
