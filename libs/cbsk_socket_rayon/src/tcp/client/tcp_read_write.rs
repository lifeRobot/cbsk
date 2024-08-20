use std::net::TcpStream;

/// tcp stream
#[derive(Default)]
pub struct TcpReadWrite {
    /// tcp stream
    pub tcp_stream: Option<TcpStream>,
}

/// custom method
impl TcpReadWrite {
    /// set tcp stream
    pub fn set_stream(&mut self, tcp_stream: TcpStream) {
        self.tcp_stream = Some(tcp_stream);
    }

    /// set none
    pub fn set_none(&mut self) {
        self.tcp_stream = None;
    }
}
