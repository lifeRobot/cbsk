#[cfg(all(not(feature = "log_pack"), feature = "dir_pack"))]
use std::fs::DirEntry;
use std::io::Write;
use std::path::Path;
#[cfg(all(not(feature = "log_pack"), feature = "dir_pack"))]
use std::path::PathBuf;
use std::sync::Arc;
use cbsk_base::log;
use zip::result::ZipResult;
use zip::write::SimpleFileOptions;
use crate::model::log_path::LogPath;

/// log zip packer
#[derive(Default)]
pub struct ZipPacker {
    /// if pack end will call this function
    pub pack_end: Option<Box<dyn Fn(String) + Sync + Send>>,
}

/// support packer
impl super::Packer for ZipPacker {
    #[cfg(feature = "log_pack")]
    fn pack(&self, split_name: String, split_path: String, log_path: Arc<LogPath>) {
        if let Err(e) = self.try_pack(split_name, split_path, log_path.clone()) {
            // printing logs here may not be meaningful
            log::error!("pack log fail: {e:?}");
        }
    }

    #[cfg(all(not(feature = "log_pack"), feature = "dir_pack"))]
    fn pack(&self, _split_name: String, _split_path: String, log_path: Arc<LogPath>) {
        if let Err(e) = self.dir_pack(log_path) {
            log::error!("pack log fail: {e:?}");
        }
    }
}

/// custom method
impl ZipPacker {
    /// set pack end function
    pub fn pack_end(mut self, f: impl Fn(String) + Sync + Send + 'static) -> Self {
        self.pack_end = Some(Box::new(f));
        self
    }

    /// try pack to log
    #[cfg(feature = "log_pack")]
    fn try_pack(&self, split_name: String, split_path: String, log_path: Arc<LogPath>) -> ZipResult<()> {
        // build zip file
        let zip_path = split_path.trim_end_matches(log_path.name_suffix.as_str());
        let zip_file = format!("{zip_path}.zip");
        self.file_try_pack(zip_file, split_name.as_str(), split_path.as_ref())
    }

    /// log file try pack
    fn file_try_pack(&self, zip_file: String, log_name: &str, log_file: &Path) -> ZipResult<()> {
        let file = cbsk_file::open_create_file(zip_file.as_ref())?;
        let mut z = zip::ZipWriter::new(file);
        z.start_file(log_name, SimpleFileOptions::default())?;
        let mut file = cbsk_file::just_open_file(log_file)?;
        std::io::copy(&mut file, &mut z)?;
        z.finish()?.flush()?;
        std::fs::remove_file(log_file)?;

        cbsk_base::match_some_return!(self.pack_end.as_ref(),Ok(()))(zip_file);
        Ok(())
    }
}

/// custom method/dir pack
#[cfg(all(not(feature = "log_pack"), feature = "dir_pack"))]
impl ZipPacker {
    /// try pack to log path
    fn dir_pack(&self, log_path: Arc<LogPath>) -> ZipResult<()> {
        // read log dir all file to vec
        let dir = std::fs::read_dir(log_path.dir.as_str())?;
        let file_list = dir.filter_map(|f| {
            let f = match f {
                Ok(f) => { f }
                Err(_) => { return None; }
            };

            if f.path().eq(&log_path.path) {
                return None;
            }
            Some(f)
        }).collect::<Vec<DirEntry>>();

        for file in file_list {
            self.file_pack(file, log_path.as_ref());
        }
        Ok(())
    }

    /// try pack file
    fn file_pack(&self, file: DirEntry, log_path: &LogPath) {
        // if file is not exists, return
        if !file.path().exists() { return; }

        let file_name = file.file_name();
        let file_name = file_name.to_str().unwrap_or_default();
        if file_name.ends_with(".zip") {
            if let Err(e) = self.zip_re_pack(file_name, log_path) {
                log::error!("zip re pack fail: {e:?}");
            }
            return;
        }

        if let Err(e) = self.log_re_pack(file_name, log_path) {
            log::error!("log re pack fail: {e:?}");
        }
    }

    /// zip re pack
    fn zip_re_pack(&self, file_name: &str, log_path: &LogPath) -> ZipResult<()> {
        // get log path and zip path etc
        let name_prefix = file_name.trim_end_matches(".zip");
        let temp_name = format!("{name_prefix}{}", log_path.name_suffix);
        let temp_path = format!("{}{temp_name}", log_path.dir);
        let temp_path = PathBuf::from(temp_path);
        let zip_file = format!("{}{file_name}", log_path.dir);
        // check log is not exists
        if !temp_path.exists() {
            cbsk_base::match_some_return!(self.pack_end.as_ref(),Ok(()))(zip_file);
            return Ok(());
        }

        // log is exists, do re pack
        std::fs::remove_file(zip_file.as_str())?;
        self.file_try_pack(zip_file, temp_name.as_str(), temp_path.as_path())
    }

    /// log re pack
    fn log_re_pack(&self, file_name: &str, log_path: &LogPath) -> ZipResult<()> {
        // get log path and zip path etc
        let temp_file = format!("{}{file_name}", log_path.dir);
        // if not ends with suffix, call pack end
        if !file_name.ends_with(log_path.name_suffix.as_str()) {
            cbsk_base::match_some_return!(self.pack_end.as_ref(),Ok(()))(temp_file);
            return Ok(());
        }

        // re get log path and etc
        let name_prefix = file_name.trim_end_matches(log_path.name_suffix.as_str());
        let temp_path = PathBuf::from(temp_file);
        let zip_file = format!("{}{name_prefix}.zip", log_path.dir);
        // if has zip file, return
        if Path::new(zip_file.as_str()).exists() { return Ok(()); }

        self.file_try_pack(zip_file, file_name, temp_path.as_path())
    }
}
