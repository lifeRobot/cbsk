use std::future::Future;
use cbsk_socket::cbsk_base::log;

/// cbsk connect and read data callback
pub trait CbskClientCallBack: Send + Sync + 'static {
    /// connect tcp server success will call this method
    fn conn(&self) -> impl Future<Output=()> + Send {
        log::info!("connect tcp server success");
        async {}
    }

    /// this method will be called when the tcp service is disconnected
    fn dis_conn(&self) -> impl Future<Output=()> + Send {
        log::info!("disconnect tcp server");
        async {}
    }

    /// connect tcp server fail and try connect server will call this method<br />
    /// num: number of try connect
    fn re_conn(&self, num: i32) -> impl Future<Output=()> + Send {
        log::info!("re connect to tcp server, re num is {num}");
        async {}
    }

    /// error frame
    fn error_frame(&self, error_frame: Vec<u8>) -> impl Future<Output=()> + Send {
        log::warn!("received non cbsk frame, will be discarded, error frame is: {error_frame:?}");
        async {}
    }

    /// data frame first byte is too long
    fn too_long_frame(&self, byte: u8) -> impl Future<Output=()> + Send {
        log::warn!("received cbsk frame, but first byte[{byte}] is too long");
        async {}
    }

    /// read tcp server data will call this method<br />
    /// bytes: cbsk server bytes<br />
    fn recv(&self, bytes: Vec<u8>) -> impl Future<Output=()> + Send;
}