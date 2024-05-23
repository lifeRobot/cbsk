/// once timer
pub struct Once {
    /// timer name
    pub name: String,
    /// once task
    pub once: Box<dyn FnOnce() + Send>,
}

/// custom method
impl Once {
    /// create once timer
    pub fn new(name: impl Into<String>, task: impl FnOnce() + Send + 'static) -> Self {
        Self {
            name: name.into(),
            once: Box::new(task),
        }
    }

    /*/// create default onec timer
    pub fn once(task: impl FnOnce() + Send + 'static) -> Self {
        Self::new("default", task)
    }*/
}
