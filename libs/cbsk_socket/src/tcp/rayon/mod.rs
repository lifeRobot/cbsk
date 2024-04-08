#[cfg(feature = "tcp_client")]
pub mod client;
#[cfg(feature = "tcp_server")]
pub mod server;
pub(crate) mod rayon_tcp_time_trait;

/// set min threads
#[cfg(any(feature = "tcp_client", feature = "tcp_server"))]
fn set_min_threads() -> Result<(), rayon::ThreadPoolBuildError> {
    #[cfg(all(feature = "tcp_client", not(feature = "tcp_server")))] let num = 2;
    #[cfg(all(feature = "tcp_server", not(feature = "tcp_client")))] let num = 3;
    #[cfg(all(feature = "tcp_client", feature = "tcp_server"))] let num = 5;
    let num = num * 2;

    // if rayon threads ge num, return
    if rayon::current_num_threads() >= num {
        return Ok(());
    }

    // rayon threads lt num, change global threads
    rayon::ThreadPoolBuilder::default().num_threads(num).build_global()
}
