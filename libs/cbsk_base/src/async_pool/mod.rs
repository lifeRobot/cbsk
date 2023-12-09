use cbsk_mut_data::mut_data_vec::MutDataVec;
use once_cell::sync::Lazy;
use tokio::task::JoinHandle;

/// global async pool
#[allow(non_upper_case_globals)]
pub static async_pool: Lazy<MutDataVec<JoinHandle<()>>> = Lazy::new(MutDataVec::default);
