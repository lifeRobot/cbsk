use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use cbsk_mut_data::mut_data_vec::MutDataVec;
use crossbeam_deque::Stealer;
use crate::job::Job;
use crate::task::Task;

/// the job state
#[derive(Clone)]
pub struct JobState {
    /// the job is running
    pub running: Arc<AtomicBool>,
    /// worker stealer
    pub stealer: Stealer<Task>,
    /// the job thread
    pub jobs: Arc<MutDataVec<Job>>,
}

/// custom method
impl JobState {
    /// create job state
    pub fn new(stealer: Stealer<Task>, jobs: Arc<MutDataVec<Job>>) -> Self {
        Self { running: Arc::new(AtomicBool::new(false)), stealer, jobs }
    }
}
