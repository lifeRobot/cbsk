use cbsk_timer::timer::Timer;

/// tcp server timer
pub struct TcpServerTimer {
    /// tcp server
    pub tcp_server: super::TcpServer,
}

/// support timer
impl Timer for TcpServerTimer {
    fn name(&self) -> &str {
        self.tcp_server.conf.log_head.as_str()
    }

    fn run(&self) {
        self.tcp_server.listener();
    }

    fn run_before(&self) -> bool {
        !**self.tcp_server.listening
    }
}

/// custom method
impl TcpServerTimer {
    /// create tcp server timer
    pub fn new(tcp_server: super::TcpServer) -> Self {
        Self { tcp_server }
    }

/*    /// start tcp server timer
    pub fn start(self) {
        cbsk_timer::push_timer(self);
        cbsk_timer::run();
    }*/
}
