use std::fs::File;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use cbsk_base::convert::datetime::DateTimeSerialize;
use cbsk_base::fastdate::DateTime;
use cbsk_base::log;
use cbsk_mut_data::mut_data_obj::MutDataObj;
use crate::model::log_path::LogPath;
use crate::model::log_size::LogSize;
use crate::packer::Packer;

/// file split actuator
pub struct FileSplitActuator {
    /// log file
    file: MutDataObj<File>,
    /// log path
    log_path: Arc<LogPath>,
    /// log size
    log_size: usize,
    /// write cache size, default is 512KB
    pub cache_size: usize,
    /// log now size
    now_size: MutDataObj<usize>,
    /// now cache size, if now cache size ge cache size, will be re open file
    now_cache_size: MutDataObj<usize>,
    /// log packer
    pub packer: Arc<Box<dyn Packer>>,
}

/// support actuator
impl super::Actuator for FileSplitActuator {
    fn exec(&self, record: &str) {
        self.check_file();
        self.write_log(record);
        self.split_log();
    }
}

/// custom method
impl FileSplitActuator {
    /// create file split actuator
    pub fn new(log_dir: impl Into<String>, log_size: LogSize, packer: impl Packer + 'static) -> io::Result<Self> {
        // get real log_dir and log_name, build log path
        let mut log_dir = log_dir.into();
        log_dir = log_dir.replace("\\", "/");
        let log_name = Self::log_name(log_dir.as_str());
        log_dir = log_dir.trim_end_matches(log_name.as_str()).into();
        let log_path = PathBuf::from(format!("{log_dir}{log_name}"));
        let (log_name_prefix, log_name_suffix) = Self::log_name_split(log_name);
        let log_path = LogPath::new(log_name_prefix, log_name_suffix, log_path, log_dir);

        let file = jui_file::open_create_file(log_path.path.as_path())?;
        let now_size = usize::try_from(file.metadata()?.len()).unwrap_or_default();

        Ok(Self {
            file: MutDataObj::new(file),
            log_path: log_path.into(),
            log_size: log_size.len(),
            cache_size: 512 * 1024,
            now_size: MutDataObj::new(now_size),
            now_cache_size: MutDataObj::new(0),
            packer: Arc::new(Box::new(packer)),
        })
    }

    /// check and split log file
    fn split_log(&self) {
        if *self.now_size < self.log_size {
            return;
        }

        // now size ge log size, split file
        let now = DateTime::now().yyyymmddhhmmss_n();
        let split_name = format!("{}_{now}{}", self.log_path.name_prefix, self.log_path.name_suffix);
        let split_file = format!("{}{split_name}", self.log_path.dir);
        if let Err(e) = std::fs::rename(self.log_path.path.as_path(), split_file.as_str()) {
            // printing logs here may not be meaningful
            eprintln!("rename fail: {e:?}");
            return;
        }

        // rename success, reset now size
        self.now_size.set(0);
        let packer = self.packer.clone();
        let log_path = self.log_path.clone();
        cbsk_timer::push_once_with_name("[cbsk_log split file]", move || {
            packer.pack(split_name, split_file, log_path);
        })
    }

    /// write log to file
    fn write_log(&self, record: &str) {
        let bytes = record.as_bytes();

        if let Err(e) = self.try_write_flush(bytes) {
            // printing logs here may not be meaningful
            log::error!("write fail:{e:?}");
            return;
        }

        // write success, add bytes len to now size
        self.now_size.set(self.now_size.saturating_add(bytes.len()));
        self.now_cache_size.set(self.cache_size.saturating_add(bytes.len()));

        // check if the file needs to be reopened
        if self.cache_size < *self.now_cache_size { return; }
        // reopen the file to release system cache
        if let Ok(file) = jui_file::open_create_file(self.log_path.path.as_path()) {
            self.file.set(file);
            self.now_cache_size.set(0);
        }
    }

    /// get log name
    fn log_name(log_dir: &str) -> String {
        let len = cbsk_base::match_some_return!(log_dir.rfind('/'),"temp.log".into());
        let log_name = &log_dir[(len + 1)..];
        if log_name.is_empty() {
            return "temp.log".into();
        }
        log_name.into()
    }

    /// get log prefix name
    fn log_name_split(log_name: String) -> (String, String) {
        let len = cbsk_base::match_some_return!(log_name.rfind("."),(log_name,"".into()));
        if len == 0 {
            return (log_name, "".into());
        }
        let split = log_name.split_at(len);
        (split.0.into(), split.1.into())
    }

    /// check file is exists<br />
    /// if not exists, will create
    fn check_file(&self) {
        if self.log_path.path.exists() {
            return;
        }

        // not exists, create
        if let Ok(file) = jui_file::open_create_file(self.log_path.path.as_path()) {
            if let Ok(meta) = file.metadata() {
                self.now_size.set(usize::try_from(meta.len()).unwrap_or_default());
            }
            self.now_cache_size.set(0);
            self.file.set(file);
        }
    }

    /// try write to file
    fn try_write_flush(&self, bytes: &[u8]) -> io::Result<()> {
        let mut file = self.file.as_mut();
        file.write_all(bytes)?;
        file.flush()
    }
}
