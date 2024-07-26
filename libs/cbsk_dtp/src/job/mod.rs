use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::thread;
use std::thread::{JoinHandle, Thread};
use cbsk_mut_data::mut_data_vec::MutDataVec;
use crossbeam_deque::{Steal, Stealer};
use crate::job::job_state::JobState;
use crate::task::Task;

mod job_state;

/// thread job
pub struct Job {
    /// job state
    pub state: JobState,
    /// the job inner thread
    pub handle: JoinHandle<()>,
}

/// custom method
impl Job {
    /// create one job
    pub fn create(stealer: Stealer<Task>, jobs: Arc<MutDataVec<Job>>) -> Self {
        let state = JobState::new(stealer, jobs);
        let handle = Self::spawn(state.clone());
        // builder default thread
        Self { state, handle }
    }

    /// job spawn
    fn spawn(state: JobState) -> JoinHandle<()> {
        thread::spawn(move || {
            loop {
                if let Steal::Success(s) = state.stealer.steal() {
                    s.run();
                }

                thread::park()
            }
        })
    }
}
