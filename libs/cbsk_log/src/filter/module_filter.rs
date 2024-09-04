use cbsk_base::parking_lot::RwLock;
use crate::model::cbsk_record::CbskRecord;

/// module filter<br />
/// block logs in this model
pub struct ModuleFilter {
    /// module list
    modules: RwLock<Vec<String>>,
}

/// support default
impl Default for ModuleFilter {
    fn default() -> Self {
        Self {
            modules: RwLock::new(Vec::with_capacity(1))
        }
    }
}

/// support filter
impl super::Filter for ModuleFilter {
    fn filter(&self, record: &CbskRecord) -> bool {
        self.modules.read().contains(&record.short_module_path().into())
    }
}

/// custom method
impl ModuleFilter {
    /// push one module filter
    pub fn push(self, module: impl Into<String>) -> Self {
        self.modules.write().push(module.into());
        self
    }

    /// append modules filter
    pub fn append(self, mut module_list: Vec<String>) -> Self {
        self.modules.write().append(&mut module_list);
        self
    }
}
