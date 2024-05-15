/// tcp server timer
pub struct TcpServerTimer {
    /// tcp server
    pub tcp_server: super::TcpServer,
}

/// custom method
impl TcpServerTimer {
    /// create tcp server timer
    pub fn new(tcp_server: super::TcpServer) -> Self {
        Self { tcp_server }
    }

    /// start timer
    pub fn start(self) {
        cbsk_timer::push_once(move || {
            loop {
                self.tcp_server.listener();
            }
        });
        cbsk_timer::run();
    }
}
