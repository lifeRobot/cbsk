use cbsk_base::tokio::net::tcp::OwnedWriteHalf;

/// tcp write
#[derive(Default)]
pub struct TcpWrite {
    /// tcp write
    pub write: Option<OwnedWriteHalf>,
}

/// custom method
impl TcpWrite {
    /// set write
    pub fn set_write(&mut self, write: OwnedWriteHalf) {
        self.write = Some(write);
    }

    /// set none
    pub fn set_none(&mut self) {
        self.write = None;
    }
}
