use std::future::Future;
use std::time::Duration;
use cbsk_base::once_cell::sync::Lazy;
use cbsk_base::tokio;
use cbsk_base::tokio::task::JoinHandle;
use cbsk_mut_data::mut_data_obj::MutDataObj;
use cbsk_mut_data::mut_data_vec::MutDataVec;

mod async_state;

/// global async pool
#[allow(non_upper_case_globals)]
pub static async_pool: Lazy<MutDataVec<JoinHandle<()>>> = Lazy::new(MutDataVec::default);

/// global async pool state
#[allow(non_upper_case_globals)]
static async_pool_state: Lazy<MutDataObj<async_state::AsyncState>> = Lazy::new(MutDataObj::default);

/// push async runtime
pub fn push<F: Future<Output=()> + Send + 'static>(f: F) {
    async_pool.push(tokio::spawn(f))
}

/// notify async pool to stop
pub fn stop() {
    async_pool_state.set(async_state::AsyncState::Stopping);
}

/// has the asyn pool been stopped
pub fn is_stop() -> bool {
    async_pool_state.is_stop()
}

/// listen to the async pool and wait for the end of the run
pub fn listener() -> JoinHandle<()> {
    tokio::spawn(async {
        running();

        loop {
            // if async pool state is stopping, remove and notify all async runtime
            if async_pool_state.is_stopping() {
                while let Some(handle) = async_pool.pop() {
                    handle.abort();
                }
                break;
            }

            // if async pool is empty, sleep 1 secs brfore continue loop
            if async_pool.is_empty() {
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }

            // remove finished async runtime
            let mut i = 0;
            let mut len = async_pool.len();
            while i < len {
                let handle = cbsk_base::match_some_exec!(async_pool.get(i),{
                    // if async_pool.get(i) is None, i add one, and continue loop
                    i += 1;
                    continue;
                });

                // remove handle when async runtime is finished
                if handle.is_finished() {
                    async_pool.remove(i);
                    len -= 1;
                }

                i += 1;
            }

            // sleep 1 secs
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        stopped();
    })
}

/// set async pool state is running
fn running() {
    async_pool_state.set(async_state::AsyncState::Running);
}

/// set async pool state is stop
fn stopped() {
    async_pool_state.set(async_state::AsyncState::Stop);
}
