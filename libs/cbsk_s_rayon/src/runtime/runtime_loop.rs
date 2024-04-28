/// runtime loop
pub enum RuntimeLoop {
    /// run once
    Once,
    /// run timer
    Timer,
    /// run tcp client
    #[cfg(feature = "tcp_client")]
    TcpClient,
    /// run tcp server
    #[cfg(feature = "tcp_server")]
    TcpServer,
    /// run tcp server client
    #[cfg(feature = "tcp_server")]
    TcpServerClient,
}
