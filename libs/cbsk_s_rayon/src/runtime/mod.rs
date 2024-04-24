use std::sync::Arc;
use std::thread;
use std::time::Duration;
use cbsk_base::{anyhow, log};
use cbsk_base::once_cell::sync::Lazy;
use cbsk_mut_data::mut_data_obj::MutDataObj;
use cbsk_mut_data::mut_data_ref::MutDataRef;
use cbsk_mut_data::mut_data_vec::MutDataVec;
use rayon::{ThreadPool, ThreadPoolBuildError};
#[cfg(feature = "tcp_client")]
use crate::client::TcpClient;
use crate::runtime::timer::Timer;
#[cfg(feature = "tcp_server")]
use crate::server::client::TcpServerClient;
#[cfg(feature = "tcp_server")]
use crate::server::TcpServer;

pub mod timer;
mod timer_loop;
mod timer_state;

/// global runtime
#[allow(non_upper_case_globals)]
pub(crate) static runtime: Lazy<Runtime> = Lazy::new(Runtime::default);

/// get thread pool
macro_rules! get_pool {
    () => {
        match runtime.try_get_pool() {
            Ok(pool) => { pool }
            Err(e) => {
                log::error!("try get pool fail: {e:?}");
                return;
            }
        }
    };
}

/// rayon thread pool runtime
pub(crate) struct Runtime {
    /// tcp server
    #[cfg(feature = "tcp_server")]
    pub tcp_server: Arc<MutDataVec<TcpServer>>,
    /// tcp server client
    #[cfg(feature = "tcp_server")]
    pub tcp_server_client: Arc<MutDataVec<Arc<TcpServerClient>>>,
    /// tcp client
    #[cfg(feature = "tcp_client")]
    pub tcp_client: Arc<MutDataVec<TcpClient>>,
    /// timer tasks
    timer: Arc<MutDataVec<Arc<Timer>>>,
    /// thread pool
    pool: Arc<MutDataVec<(ThreadPool, isize)>>,
    /// is global runtime running
    running: Arc<MutDataObj<bool>>,
}

/// support default
impl Default for Runtime {
    fn default() -> Self {
        let pool = MutDataVec::with_capacity(2);
        pool.push((build_pool(), 0));

        Self {
            #[cfg(feature = "tcp_server")]
            tcp_server: MutDataVec::with_capacity(2).into(),
            #[cfg(feature = "tcp_server")]
            tcp_server_client: MutDataVec::with_capacity(2).into(),
            #[cfg(feature = "tcp_client")]
            tcp_client: MutDataVec::with_capacity(2).into(),
            timer: MutDataVec::default().into(),
            pool: pool.into(),
            running: MutDataObj::new(false).into(),
        }
    }
}

/// custom method
impl Runtime {
    /// start the global runtime
    pub fn start(&self) {
        if **self.running { return; }

        self.run();
    }

    /// global runtime logic
    fn run(&self) {
        self.running.set_true();

        let pool = cbsk_base::match_some_return!(runtime.pool.get_mut(0),{
            log::error!("start global runtime fail");
            self.running.set_false();
        });
        add_spawn(pool, Box::new(|| {
            loop {
                runtime.run_timer();
                #[cfg(feature = "tcp_server")]
                runtime.run_tcp_server();
                #[cfg(feature = "tcp_server")]
                runtime.run_tcp_server_client();
                #[cfg(feature = "tcp_client")]
                runtime.run_tcp_client();

                thread::sleep(Duration::from_millis(10));
            }
        }));
    }

    /// try get pool
    fn try_get_pool(&self) -> anyhow::Result<MutDataRef<(ThreadPool, isize)>> {
        for pool in self.pool.iter_mut() {
            if pool.1 < 10 {
                return Ok(pool);
            }
        }

        // if not pool, build once and return
        self.pool.push((try_build_pool().map_err(|e| { anyhow::anyhow!("create new thread pool fail: {e:?}") })?, 0));
        self.pool.get_mut(self.pool.len() - 1).ok_or_else(|| { anyhow::anyhow!("try get thread pool fail") })
    }
}

/// run tcp client/server business
impl Runtime {
    /// run timer
    fn run_timer(&self) {
        for (i, t) in self.timer.iter().enumerate() {
            // if task is end, remove task and return
            if *t.end {
                self.timer.remove(i);
                return;
            }

            // if not ready, return
            if !t.ready() {
                return;
            }

            // if task is can't run
            let t = t.clone();
            add_spawn(get_pool!(), Box::new(move || {
                t.run();
            }))
        }
    }

    /// run tcp client logic
    #[cfg(feature = "tcp_client")]
    fn run_tcp_client(&self) {
        for (i, tc) in self.tcp_client.iter().enumerate() {
            let tc = tc.clone();
            // if tcp client dis connectioned
            if !tc.is_connected() {
                // not conn and not first and not re conn, remove this tcp client
                if !tc.state.first && !tc.conf.reconn.enable {
                    runtime.tcp_client.remove(i);
                    return;
                }
                add_spawn(get_pool!(), Box::new(move || {
                    tc.conn();
                }));
                continue;
            }

            if tc.state.reading {
                tc.check_read_finished();
                continue;
            }
            // if tcp not reading
            add_spawn(get_pool!(), Box::new(move || {
                tc.read();
            }))
        }
    }

    /// run tcp server logic
    #[cfg(feature = "tcp_server")]
    fn run_tcp_server(&self) {
        self.tcp_server.iter().for_each(|ts| {
            if **ts.listening { return; }

            let ts = ts.clone();
            add_spawn(get_pool!(), Box::new(move || {
                ts.listener();
            }));
        })
    }

    /// run tcp server client logic
    #[cfg(feature = "tcp_server")]
    fn run_tcp_server_client(&self) {
        for (i, tc) in self.tcp_server_client.iter().enumerate() {
            // if dis connection, remove and return
            if !**tc.connecting {
                runtime.tcp_server_client.remove(i);
                return;
            }

            let tc = tc.clone();
            if **tc.reading {
                tc.check_read_finished(tc.clone());
                continue;
            }

            add_spawn(get_pool!(), Box::new(move || {
                tc.read(tc.clone());
            }))
        }
    }
}

/// add one spawn
fn add_spawn(mut pool: MutDataRef<(ThreadPool, isize)>, spawn: Box<dyn Fn() + Send>) {
    pool.1 += 1;
    pool.clone().0.spawn(move || {
        spawn();
        pool.1 -= 1;
    });
}

/// build thread pool
fn build_pool() -> ThreadPool {
    try_build_pool().expect("create thread pool fail")
}

/// try build thread pool
fn try_build_pool() -> Result<ThreadPool, ThreadPoolBuildError> {
    rayon::ThreadPoolBuilder::default().num_threads(10).build()
}

/// push once task<br />
/// please do not use dead loops in tasks
pub fn push_once(task: impl Fn(&Timer) + Sync + Send + 'static) {
    runtime.timer.push(Timer::once(task).into())
}

/// push interval task<br />
/// please do not use dead loops in tasks
pub fn push_task(interval: Duration, task: impl Fn(&Timer) + Sync + Send + 'static) {
    runtime.timer.push(Timer::new(interval, task).into())
}

/// start runtime
pub fn start() {
    runtime.start();
}
