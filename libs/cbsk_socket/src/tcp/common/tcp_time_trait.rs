use fastdate::DateTime;

/// TCP time control related trait
pub trait TcpTimeTrait {
    /// set the last time the data was received for tcp
    fn set_recv_time(&self, time: i64);

    /// get the last time the data was received for tcp
    fn get_recv_time(&self) -> i64;

    /// set tcp last read timeout
    fn set_timeout_time(&self, time: i64);

    /// get tcp last read timeout
    fn get_timeout_time(&self) -> i64;

    /// set recv time and timeout time to now
    fn set_now(&self) {
        self.set_recv_time_now();
        self.set_timeout_time_now();
    }

    /// set recv time to now
    fn set_recv_time_now(&self) {
        self.set_recv_time(Self::now());
    }

    /// set timeout time to now
    fn set_timeout_time_now(&self) {
        self.set_timeout_time(Self::now())
    }

    /// get now unix_timestamp_millis
    fn now() -> i64 {
        DateTime::now().unix_timestamp_millis()
    }


    /// set is wait callback
    fn set_wait_callback(&self, is_wait: bool);

    /// get is wait callback
    fn get_wait_callback(&self) -> bool;

    /// need wait callback finish business logic
    fn wait_callback(&self) {
        self.set_wait_callback(true);
    }

    /// callback the business logic has been completed
    fn finish_callback(&self) {
        self.set_wait_callback(false);
        // if finish callback, default timeout is now
        self.set_timeout_time_now();
    }
}