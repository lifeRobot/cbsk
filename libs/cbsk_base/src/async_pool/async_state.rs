/// async pool state
pub enum AsyncState {
    Prepare,
    Running,
    Stopping,
    Stop,
}

/// support default is prepare
impl Default for AsyncState {
    fn default() -> Self {
        Self::Prepare
    }
}

/// custom method
impl AsyncState {
    pub fn is_stop(&self) -> bool {
        matches!(self,Self::Stop)
    }
}
