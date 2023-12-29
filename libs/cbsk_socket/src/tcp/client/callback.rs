use std::future::Future;
use cbsk_base::log;

/// tcp connect and read data callback
pub trait TcpClientCallBack: Send + Sync + 'static {
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

    /// read tcp server data will call this method<br />
    /// bytes: tcp server bytes<br />
    /// return Vec<u8>: If you think the data length is insufficient, you can return the data for data merging,
    /// if data normal, you should be return Vec::new() or vec![]
    fn recv(&self, bytes: Vec<u8>) -> impl Future<Output=Vec<u8>> + Send;
}