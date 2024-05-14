use std::io;
use cbsk_base::log::LevelFilter;
use cbsk_mut_data::mut_data_vec::MutDataVec;
use crate::actuator::Actuator;
use crate::actuator::console_actuator::ConsoleActuator;
use crate::actuator::file_split_actuator::FileSplitActuator;
use crate::filter::Filter;
use crate::format::default_format::DefaultFormat;
use crate::format::LogFormat;
use crate::model::cbsk_record::CbskRecord;
use crate::model::log_size::LogSize;
use crate::packer::Packer;

/// cbsk log config
pub struct Config {
    /// log level filter
    pub level: LevelFilter,
    /// log actuator
    pub actuators: MutDataVec<Box<dyn Actuator>>,
    /// log format
    pub format: Box<dyn LogFormat>,
    /// log filter
    pub filter: MutDataVec<Box<dyn Filter>>,
    /// log max refresh items, min and default is 100<br />
    /// if the number of log cache entries is greater than max_refresh, each time max_refresh entries are obtained and given to actuator for processing
    pub max_refresh: usize,
}

/// support default
impl Default for Config {
    fn default() -> Self {
        Self {
            level: LevelFilter::Info,
            actuators: MutDataVec::with_capacity(1),
            format: Box::new(DefaultFormat::default()),
            filter: MutDataVec::with_capacity(1),
            max_refresh: 100,
        }
    }
}

/// custom method
impl Config {
    /// set log level
    pub fn level(mut self, level: LevelFilter) -> Self {
        self.level = level;
        self
    }

    /// push log filter
    pub fn push_filter(self, filter: impl Filter + 'static) -> Self {
        self.filter.push(Box::new(filter));
        self
    }

    /// append log filters
    pub fn append_filter(self, mut filter_list: Vec<Box<dyn Filter>>) -> Self {
        self.filter.append(&mut filter_list);
        self
    }

    /// is filter log
    pub(crate) fn filter(&self, record: &CbskRecord) -> bool {
        for filter in self.filter.iter() {
            if filter.filter(record) {
                return true;
            }
        }

        false
    }

    /// set log format
    pub fn format(mut self, format: impl LogFormat + 'static) -> Self {
        self.format = Box::new(format);
        self
    }

    /// set max refresh
    pub fn max_refresh(mut self, max_refresh: usize) -> Self {
        if max_refresh > 100 {
            self.max_refresh = max_refresh;
        }
        self
    }

    /// output logs in the console
    pub fn console(self) -> Self {
        self.actuators.push(Box::new(ConsoleActuator {}));
        self
    }

    /// try add output logs in file split
    pub fn try_file_split(self, log_dir: impl Into<String>, log_size: LogSize, packer: impl Packer + 'static) -> io::Result<Self> {
        self.actuators.push(Box::new(FileSplitActuator::new(log_dir, log_size, packer)?));
        Ok(self)
    }

    /// output logs in file split<br />
    /// ## Panic
    /// if create log file fail, will panic
    pub fn file_split(self, log_dir: impl Into<String>, log_size: LogSize, packer: impl Packer + 'static) -> Self {
        self.try_file_split(log_dir, log_size, packer).expect("create file split fail")
    }
}
