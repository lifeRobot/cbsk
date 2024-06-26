use std::sync::Arc;
use cbsk_mut_data::mut_data_obj::MutDataObj;
use cbsk_socket::tcp::common::tcp_time_trait::TcpTimeTrait;
use cbsk_timer::timer::Timer;

/// tcp server client timer
pub struct TcpServerClientTimer {
    /// tcp client
    pub tcp_client: Arc<super::client::TcpServerClient>,
    /// tcp client timer is need end?
    pub end: MutDataObj<bool>,
}

/// support timer
impl Timer for TcpServerClientTimer {
    fn name(&self) -> &str {
        self.tcp_client.log_head.as_str()
    }

    fn run(&self) {
        self.tcp_client.read(self.tcp_client.clone());
    }

    fn run_before(&self) -> bool {
        let tc = self.tcp_client.as_ref();

        // if dis connection, remove and return
        if !**tc.connecting {
            self.end.set_true();
            return false;
        }

        if **tc.reading {
            tc.check_read_finished(self.tcp_client.clone());
            return false;
        }

        // if net buf is empty, and now and timeout time should not exceed 100 milliseconds, return false
        // note that this approach may result in data not being real-time
        if self.tcp_client.next_buf.is_empty() {
            let now = super::TcpServerClient::now();
            let timeout_time = self.tcp_client.get_timeout_time();
            let diff = now - timeout_time;
            // fixed the issue of returning false after a time jump
            if diff > 0 && diff <= 100 {
                return false;
            }
        }

        true
    }

    fn ended(&self) -> bool {
        *self.end
    }
}

/// custom method
impl TcpServerClientTimer {
    /// create tcp server client timer
    pub fn new(tcp_client: Arc<super::client::TcpServerClient>) -> Self {
        Self {
            tcp_client,
            end: MutDataObj::default(),
        }
    }

    /*/// start tcp server client timer
    pub fn start(self) {
        cbsk_timer::push_timer(self);
        cbsk_timer::run();
    }*/
}
