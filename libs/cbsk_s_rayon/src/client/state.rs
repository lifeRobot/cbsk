/// tcp client state
pub struct TcpState {
    /// is first conn to tcp server
    /// default is true
    pub first: bool,
    /// re connection tcp server num
    pub re_num: i32,
    /// last re connection tcp server time
    pub last_re_time: i64,
    /// the tcp client is reading
    pub reading: bool,
    /// the tcp client is connecting
    pub connecting: bool,
}

/// support default
impl Default for TcpState {
    fn default() -> Self {
        Self {
            first: true,
            re_num: 0,
            last_re_time: 0,
            reading: false,
            connecting: false,
        }
    }
}
