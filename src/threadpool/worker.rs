//! A thread worker

use crate::{error::Error, log_error, log_info, threadpool::StaticFn};
use std::{
    sync::{
        mpsc::{self, Receiver, SyncSender},
        Arc,
    },
    thread::Builder,
};

/// A thread
pub struct Worker<T, const STACK_SIZE: usize> {
    /// The incoming job
    job: Receiver<StaticFn<T>>,
    /// The "seed" of the `job`-queue to reinsert it into the threadpool
    job_seed: SyncSender<StaticFn<T>>,
    /// The owning threadpool
    threadpool: SyncSender<SyncSender<StaticFn<T>>>,
    /// The worker counter
    _worker_counter: Arc<()>,
}
impl<T, const STACK_SIZE: usize> Worker<T, STACK_SIZE> {
    /// Spawns a new worker and returns it's job queue
    pub fn spawn(
        threadpool: &SyncSender<SyncSender<StaticFn<T>>>,
        worker_counter: &Arc<()>,
    ) -> Result<SyncSender<StaticFn<T>>, Error>
    where
        T: Send + 'static,
    {
        // Create the job queue and init self
        let (job_seed, job) = mpsc::sync_channel(1);
        let this = Self {
            job,
            job_seed: job_seed.clone(),
            threadpool: threadpool.clone(),
            _worker_counter: worker_counter.clone(),
        };

        // Spawn the thread
        let builder = Builder::new().stack_size(STACK_SIZE).name("threadpool worker thread".to_string());
        if let Err(e) = builder.spawn(|| this.runloop()) {
            log_error!("Failed to spawn new worker thread");
            return Err(e.into());
        };
        Ok(job_seed)
    }

    /// The worker runloop
    fn runloop(self) {
        loop {
            // Read the job from the queue
            let Ok(job) = self.job.recv() else {
                log_info!("Unable to receive next job, stopping worker thread");
                return;
            };

            // Execute job
            let StaticFn { fn_, context } = job;
            fn_(context);

            // Mark us as ready to receive jobs
            let jobs_seed = self.job_seed.clone();
            if self.threadpool.try_send(jobs_seed).is_err() {
                log_info!("Unable to requeue worker, stopping worker thread");
                return;
            }
        }
    }
}
