use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::LazyLock;
use std::thread;
use std::time::Duration;
use cbsk_base::tokio;
use cbsk_base::tokio::task::JoinHandle;
use cbsk_log::config::Config;
use cbsk_log::model::cbsk_record::CbskRecord;
use crate::runtime::cache::log_cache;

pub mod cache;

/// log timer is running
#[allow(non_upper_case_globals)]
static running: LazyLock<AtomicBool> = LazyLock::new(AtomicBool::default);

/// log runtime
#[derive(Default)]
pub struct LogRuntime {}

/// custom method
impl LogRuntime {
    /// start log logic
    pub fn start(&self, config: Config) -> JoinHandle<()> {
        tokio::spawn(async {
            Self::run(config).await;
        })
    }

    /// log logic
    async fn run(config: Config) {
        loop {
            let mut format_str = Self::recv(&config).await;
            Self::recv_refresh(&mut format_str, &config);
            Self::actuators(format_str.as_str(), &config);
        }
    }

    /// log to actuators
    fn actuators(format_str: &str, config: &Config) {
        for at in config.actuators.iter() {
            at.exec(format_str);
        }
        running.store(false, Ordering::Release);
    }

    /// recv all/max_refresh log
    fn recv_refresh(format_str: &mut String, config: &Config) {
        let mut refersh = 1;
        let mut write = log_cache.recv.write();
        while let Ok(record) = write.try_recv() {
            // filter
            if Self::filter(&record, config) { continue; }

            // format
            format_str.push_str(config.format.format(&record).as_str());
            refersh += 1;
            if refersh < config.max_refresh {
                break;
            }
        }
    }

    /// wait log
    async fn recv(config: &Config) -> String {
        let mut format_str = String::with_capacity(10);
        loop {
            let record =
                match tokio::time::timeout(Duration::from_secs(1), log_cache.recv.write().recv()).await.unwrap_or_default() {
                    Some(record) => { record }
                    None => { continue; }
                };

            running.store(true, Ordering::Release);
            // not filter
            if !Self::filter(&record, config) {
                format_str.push_str(config.format.format(&record).as_str());
                break;
            }
        }

        format_str
    }

    /// is filter log
    fn filter(record: &CbskRecord, config: &Config) -> bool {
        for filter in config.filter.iter() {
            if filter.filter(record) {
                return true;
            }
        }
        false
    }

    /// wait log flush
    pub fn wait_flush() {
        while !cache::is_empty() || running.load(Ordering::Acquire) {
            thread::sleep(Duration::from_millis(100));
        }
    }
}