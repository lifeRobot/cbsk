use cbsk_base::log;

/// cbsk connect and read data callback
pub trait CbskClientCallBack: Send + Sync + 'static {
    /// connect tcp server success will call this method
    fn conn(&self) {
        log::info!("connect tcp server success");
    }

    /// this method will be called when the tcp service is disconnected
    fn dis_conn(&self) {
        log::info!("disconnect tcp server");
    }

    /// connect tcp server fail and try connect server will call this method<br />
    /// num: number of try connect
    fn re_conn(&self, num: i32) {
        log::info!("re connect to tcp server, re num is {num}");
    }

    /// error frame
    fn error_frame(&self, error_frame: Vec<u8>) {
        log::warn!("received non cbsk frame, will be discarded, error frame is: {error_frame:?}");
    }

    /// data frame first byte is too long
    fn too_long_frame(&self, byte: u8) {
        log::warn!("received cbsk frame, but first byte[{byte}] is too long");
    }

    /// read tcp server data will call this method<br />
    /// bytes: cbsk server bytes<br />
    fn recv(&self, bytes: Vec<u8>);
}