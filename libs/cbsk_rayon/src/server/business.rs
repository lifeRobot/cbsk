use std::sync::Arc;
use cbsk::{business, data};
#[cfg(feature = "debug_mode")]
use cbsk_base::log;
use cbsk_socket_rayon::tcp::server::callback::TcpServerCallBack;
use cbsk_socket_rayon::tcp::server::client::TcpServerClient;
use crate::server::callback::CbskServerCallBack;
use crate::server::client::CbskServerClient;

/// support tcp server callback
pub struct CbskServerBusines<C: CbskServerCallBack> {
    /// the cbsk first frame<br />
    /// Used to determine if it is cbsk data
    pub header: Arc<Vec<u8>>,
    /// business callback
    pub cb: Arc<C>,
    /// internal log name, used for log printing
    pub log_head: String,
}

/// custom method
impl<C: CbskServerCallBack> CbskServerBusines<C> {
    /// new business
    pub fn new(cb: Arc<C>) -> Self {
        Self { cb, header: data::default_header().into(), log_head: String::new() }
    }

    /// new business, custom header frame
    pub fn new_with_head(cb: Arc<C>, mut header: Vec<u8>) -> Self {
        // if header is empty, set default
        if header.is_empty() {
            header = data::default_header()
        }
        Self { cb, header: header.into(), log_head: String::new() }
    }
}

/// support tcp server callback
impl<C: CbskServerCallBack> TcpServerCallBack for CbskServerBusines<C> {
    fn conn(&self, client: Arc<TcpServerClient>) {
        self.cb.conn(CbskServerClient::new(self.header.clone(), client).into());
    }

    fn dis_conn(&self, client: Arc<TcpServerClient>) {
        self.cb.dis_conn(CbskServerClient::new(self.header.clone(), client).into());
    }

    fn recv(&self, mut bytes: Vec<u8>, client: Arc<TcpServerClient>) -> Vec<u8> {
        let cbsk_server_client = Arc::new(CbskServerClient::new(self.header.clone(), client));

        // TODO can the following code be optimized? There are too many if and loop
        #[cfg(feature = "debug_mode")]
        log::info!("{} start recv loop", self.log_head);
        loop {
            let mut verify_data = business::verify(bytes, &self.header);
            #[cfg(feature = "debug_mode")] {
                log::info!("{} cbsk recv loop", self.log_head);
                log::info!("{} error_frame len is {}", self.log_head, verify_data.error_frame.len());
                log::info!("{} too_short_frame len is {}", self.log_head, verify_data.too_short_frame.len());
                log::info!("{} data_frame len is {}", self.log_head, verify_data.data_frame.len());
                log::info!("{} next_verify_frame len is {}", self.log_head, verify_data.next_verify_frame.len());
            }

            if !verify_data.error_frame.is_empty() {
                self.cb.error_frame(verify_data.error_frame, cbsk_server_client.clone());
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
                        self.cb.too_long_frame(too_long, cbsk_server_client.clone());
                    }

                    // analysis success, call cb.recv
                    if !analysis_data.data_frame.is_empty() {
                        self.cb.recv(analysis_data.data_frame, cbsk_server_client.clone());
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
