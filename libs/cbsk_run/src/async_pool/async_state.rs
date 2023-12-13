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
    /// has the async pool state been stoped
    pub fn is_stop(&self) -> bool {
        matches!(self,Self::Stop)
    }

    /// is the async pool state stopping
    pub fn is_stopping(&self) -> bool {
        matches!(self,Self::Stopping)
    }
}
