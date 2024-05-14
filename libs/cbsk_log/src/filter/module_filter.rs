use cbsk_mut_data::mut_data_vec::MutDataVec;
use crate::model::cbsk_record::CbskRecord;

/// module filter<br />
/// block logs in this model
pub struct ModuleFilter {
    /// module list
    pub modules: MutDataVec<String>,
}

/// support default
impl Default for ModuleFilter {
    fn default() -> Self {
        Self {
            modules: MutDataVec::with_capacity(1)
        }
    }
}

/// support filter
impl super::Filter for ModuleFilter {
    fn filter(&self, record: &CbskRecord) -> bool {
        self.modules.contains(&record.short_module_path().into())
    }
}

/// custom method
impl ModuleFilter {
    /// push one module filter
    pub fn push(self, module: impl Into<String>) -> Self {
        self.modules.push(module.into());
        self
    }

    /// append modules filter
    pub fn append(self, mut module_list: Vec<String>) -> Self {
        self.modules.append(&mut module_list);
        self
    }
}
