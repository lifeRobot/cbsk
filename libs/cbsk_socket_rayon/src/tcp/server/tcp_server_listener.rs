use std::net::TcpListener;

/// tcp server listener
#[derive(Default)]
pub struct TcpServerListener {
    /// listener
    pub listener: Option<TcpListener>,
}

/// custom method
impl TcpServerListener {
    /// set listener
    pub fn set_listener(&mut self, listener: TcpListener) {
        self.listener = Some(listener);
    }

    /// set none
    pub fn set_none(&mut self) {
        self.listener = None
    }
}
