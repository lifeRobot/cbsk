use cbsk_mut_data::mut_data_obj::MutDataObj;
use cbsk_socket::tcp::common::sync::tcp_write_trait::TcpWriteTrait;
use cbsk_socket::tcp::common::tcp_time_trait::TcpTimeTrait;
use cbsk_timer::timer::Timer;
use crate::client::timer_state::TimerState;

/// tcp client timer
pub struct TcpClientTimer {
    /// tcp client
    pub tcp_client: super::TcpClient,
    /// tcp client timer is need end?
    pub end: MutDataObj<bool>,
    /// timer state
    pub state: MutDataObj<TimerState>,
}

/// support timer
impl Timer for TcpClientTimer {
    fn name(&self) -> &str {
        self.tcp_client.get_log_head()
    }

    fn run(&self) {
        match self.state.as_ref() {
            TimerState::Conn => {
                self.tcp_client.conn();
                self.tcp_client.state.as_mut().connecting = false;
            }
            TimerState::Read => { self.tcp_client.read() }
        }
    }

    fn run_before(&self) -> bool {
        let tc = &self.tcp_client;
        if tc.state.connecting {
            return false;
        }

        if !tc.is_connected() {
            if !tc.state.first && !tc.conf.reconn.enable {
                self.end.set_true();
                return false;
            }

            // check neet conn
            let diff = u128::try_from(super::TcpClient::now() - self.tcp_client.state.last_re_time).unwrap_or_default();
            if diff < self.tcp_client.conf.reconn.time.as_millis() {
                // diff lt reconn wait time, not need for conn
                return false;
            }

            // need conn
            self.state.set(TimerState::Conn);
            return true;
        }

        if tc.state.reading {
            // need check read finished
            self.tcp_client.check_read_finished();
            return false;
        }

        // if net buf is empty, and now and timeout time should not exceed 100 milliseconds, return false
        // note that this approach may result in data not being real-time
        if self.tcp_client.next_buf.is_empty() {
            let now = super::TcpClient::now();
            let timeout_time = self.tcp_client.get_timeout_time();
            let diff = now - timeout_time;
            if diff <= 100 {
                return false;
            }
        }
        // just run read
        self.state.set(TimerState::Read);
        true
    }

    fn ended(&self) -> bool {
        *self.end
    }
}

/// custom method
impl TcpClientTimer {
    /// create tcp client timer
    pub fn new(tcp_client: super::TcpClient) -> Self {
        Self {
            tcp_client,
            end: MutDataObj::default(),
            state: MutDataObj::default(),
        }
    }

    /*    /// start tcp client timer
        pub fn start(self) {
            cbsk_timer::push_timer(self);
            cbsk_timer::run();
        }*/
}
