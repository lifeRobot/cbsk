use std::ops::Deref;
use std::sync::atomic::{AtomicBool, Ordering};
use cbsk_base::parking_lot::RwLock;
use cbsk_socket::tcp::common::time_trait::TimeTrait;
use cbsk_timer::timer::Timer;
use crate::tcp::client::timer_state::TimerState;
use crate::tcp::common::tcp_write_trait::TcpWriteTrait;

/// tcp client timer
pub struct TcpClientTimer {
    /// tcp client
    pub tcp_client: super::TcpClient,
    /// tcp client timer is need end?
    pub end: AtomicBool,
    /// timer state
    pub state: RwLock<TimerState>,
}

/// support timer
impl Timer for TcpClientTimer {
    fn name(&self) -> &str {
        self.tcp_client.get_log_head()
    }

    fn run(&self) {
        match self.state.read().deref() {
            TimerState::Conn => {
                self.tcp_client.conn();
                self.tcp_client.state.write().connecting = false;
            }
            TimerState::Read => {
                self.tcp_client.read()
            }
        }
    }

    fn run_before(&self) -> bool {
        let tc = &self.tcp_client;
        let state = tc.state.read();
        if state.connecting {
            return false;
        }

        if !tc.is_connected() {
            if !state.first && !tc.conf.reconn.enable {
                self.end.store(true, Ordering::Relaxed);
                return false;
            }

            // check neet conn
            let diff = u128::try_from(super::TcpClient::now() - state.last_re_time).unwrap_or_default();
            if diff < self.tcp_client.conf.reconn.time.as_millis() {
                // diff lt reconn wait time, not need for conn
                return false;
            }

            // need conn
            *self.state.write() = TimerState::Conn;
            return true;
        }

        if state.reading {
            // need check read finished
            self.tcp_client.check_read_finished();
            return false;
        }

        // if net buf is empty, and now and timeout time should not exceed 100 milliseconds, return false
        // note that this approach may result in data not being real-time
        if self.tcp_client.next_buf.read().is_empty() {
            let now = super::TcpClient::now();
            let timeout_time = self.tcp_client.get_timeout_time();
            let diff = now - timeout_time;
            // fixed the issue of returning false after a time jump
            if diff > 0 && diff <= 100 {
                return false;
            }
        }
        // just run read
        *self.state.write() = TimerState::Read;
        true
    }

    fn ended(&self) -> bool {
        self.end.load(Ordering::Relaxed)
    }
}

/// custom method
impl TcpClientTimer {
    /// create tcp client timer
    pub fn new(tcp_client: super::TcpClient) -> Self {
        Self {
            tcp_client,
            end: AtomicBool::default(),
            state: RwLock::default(),
        }
    }
}
