use std::thread;
use std::time::Duration;
use cbsk_base::once_cell::sync::Lazy;
use cbsk_mut_data::mut_data_obj::MutDataObj;
use crate::config::Config;
use crate::runtime::cache::log_cache;

pub mod cache;

/// global log config
#[allow(non_upper_case_globals)]
pub static log_conf: Lazy<MutDataObj<Config>> = Lazy::new(MutDataObj::default);

/// log timer is running
#[allow(non_upper_case_globals)]
static running: Lazy<MutDataObj<bool>> = Lazy::new(MutDataObj::default);

/// log runtime
#[derive(Default)]
pub struct LogRuntime {}

/// custom method
impl LogRuntime {
    /// start log timer
    pub fn start(&self) {
        cbsk_timer::push_once_with_name("cbsk_log", Self::run);
        cbsk_timer::run();
    }

    /// log logic
    fn run() {
        loop {
            let mut format_str = String::with_capacity(10);
            let mut refersh = 0;
            if log_cache.recv.is_empty() {
                if let Ok(record) = log_cache.recv.recv() {
                    // not filter
                    if !log_conf.filter(&record) {
                        format_str.push_str(log_conf.format.format(&record).as_str());
                        refersh += 1;
                    }
                }
            }

            // read all
            running.set_true();
            while let Ok(record) = log_cache.recv.try_recv() {
                // filter
                if log_conf.filter(&record) {
                    continue;
                }

                // format
                format_str.push_str(log_conf.format.format(&record).as_str());
                refersh += 1;
                if refersh < log_conf.max_refresh {
                    break;
                }
            }

            // to actuator
            for at in log_conf.actuators.iter() {
                at.exec(format_str.as_str());
            }
            running.set_false();
        }
    }

    /// wait log flush
    pub fn wait_flush() {
        while !cache::is_empty() || **running {
            thread::sleep(Duration::from_millis(100));
        }
    }
}
