use std::time::Duration;

/// socket reconnected config
#[derive(Default)]
pub struct SocketReConn {
    /// enable reconn
    pub enable: bool,
    /// reconn wait time
    pub time: Duration,
}


/// custom method
impl SocketReConn {
    /// create socket reconnected config
    pub fn new(enable: bool, time: Duration) -> Self {
        Self { enable, time }
    }

    /// create enable socket reconnect config
    pub fn enable(time: Duration) -> Self {
        Self::new(true, time)
    }
}

