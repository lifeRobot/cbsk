use std::sync::Arc;
use cbsk_base::async_trait::async_trait;
use cbsk_socket::tcp::client::callback::TcpClientCallBack;
use crate::{business, data};
use crate::client::callback::CbskClientCallBack;

/// support tcp client callback
pub struct CbskClientBusiness<C: CbskClientCallBack> {
    /// the cbsk first frame<br />
    /// Used to determine if it is cbsk data
    pub header: Arc<Vec<u8>>,
    /// business callback
    pub cb: Arc<C>,
}

/// custom method
impl<C: CbskClientCallBack> CbskClientBusiness<C> {
    /// new business
    pub fn new(cb: Arc<C>) -> Self {
        Self { cb, header: data::default_header().into() }
    }

    /// new business, custom header frame
    pub fn new_with_head(cb: Arc<C>, mut header: Vec<u8>) -> Self {
        // if header is empty, set default
        if header.is_empty() {
            header = data::default_header()
        }
        Self { cb, header: header.into() }
    }
}

/// support tcp client callback
#[async_trait]
impl<C: CbskClientCallBack> TcpClientCallBack for CbskClientBusiness<C> {
    async fn conn(&self) {
        self.cb.conn().await;
    }

    async fn dis_conn(&self) {
        self.cb.dis_conn().await;
    }

    async fn re_conn(&self, num: i32) {
        self.cb.re_conn(num).await
    }

    async fn recv(&self, mut bytes: Vec<u8>) -> Vec<u8> {
        // TODO can the following code be optimized? There are too many if and loop
        loop {
            let mut verify_data = business::verify(bytes, &self.header);
            if !verify_data.error_frame.is_empty() {
                self.cb.error_frame(verify_data.error_frame).await;
            }

            // if has too short frame, wait next tcp read
            if !verify_data.too_short_frame.is_empty() {
                return verify_data.too_short_frame;
            }

            // verify success, perform data analysis
            if !verify_data.data_frame.is_empty() {
                loop {
                    let analysis_data = business::analysis(verify_data.data_frame, &self.header);

                    if let Some(too_long) = analysis_data.too_long_byte {
                        self.cb.too_long_frame(too_long).await;
                    }

                    // analysis success, call cb.recv
                    if !analysis_data.data_frame.is_empty() {
                        self.cb.recv(analysis_data.data_frame).await;
                    }

                    // if has next verify, change verify_data.next_verify_frame and break current loop
                    if !analysis_data.next_verify_frame.is_empty() {
                        verify_data.next_verify_frame = analysis_data.next_verify_frame;
                        break;
                    }

                    // if has too short frame, wait next tcp read
                    if !analysis_data.too_short_frame.is_empty() {
                        return analysis_data.too_short_frame;
                    }

                    // analysis logic over, break current loop
                    break;
                }
            }

            // if has next verify, change bytes and go to next loop
            if !verify_data.next_verify_frame.is_empty() {
                bytes = verify_data.next_verify_frame;
                continue;
            }

            // verify logic over, break loop
            break;
        }

        // default return empty data
        Vec::new()
    }
}
