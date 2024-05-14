use std::io::Write;
use std::sync::Arc;
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
    fn pack(&self, split_name: String, split_path: String, log_path: Arc<LogPath>) {
        if let Err(e) = self.try_pack(split_name, split_path, log_path) {
            // printing logs here may not be meaningful
            eprintln!("pack log fail: {e:?}")
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
    fn try_pack(&self, split_name: String, split_path: String, log_path: Arc<LogPath>) -> ZipResult<()> {
        // build zip file
        let zip_path = split_path.trim_end_matches(log_path.name_suffix.as_str());
        let zip_file = format!("{zip_path}.zip");
        let file = jui_file::open_create_file(zip_file.as_ref())?;
        let mut z = zip::ZipWriter::new(file);
        z.start_file(split_name.as_str(), SimpleFileOptions::default())?;
        let mut file = jui_file::just_open_file(split_path.as_ref())?;
        std::io::copy(&mut file, &mut z)?;
        z.flush()?;
        std::fs::remove_file(split_path.as_str())?;

        if let Some(pack_end) = self.pack_end.as_ref() {
            pack_end(zip_file);
        }
        Ok(())
    }
}
