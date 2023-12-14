use cbsk_base::async_trait::async_trait;

/// tcp connect and read data callback
#[async_trait]
pub trait TcpClientCallBack: Send + Sync + 'static {
    /// connect tcp server success will call this method
    async fn conn(&self);

    /// this method will be called when the tcp service is disconnected
    async fn dis_conn(&self);

    /// connect tcp server fail and try connect server will call this method<br />
    /// num: number of try connect
    async fn re_conn(&self, num: i32);

    /// read tcp server data will call this method<br />
    /// bytes: tcp server bytes<br />
    /// return Vec<u8>: If you think the data length is insufficient, you can return the data for data merging,
    /// if data normal, you should be return Vec::new() or vec![]
    async fn recv(&self, bytes: Vec<u8>) -> Vec<u8>;
}