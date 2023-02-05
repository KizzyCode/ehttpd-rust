//! A thread worker

use crate::{
    error::Error,
    threadpool::{counter::Counter, Executable},
};
use flume::Receiver;
use std::{sync::Arc, thread::Builder, time::Duration};

/// A thread
pub struct Worker<T, const STACK_SIZE: usize, const TIMEOUT: u64 = 5> {
    /// The receiving half of the job-queue
    queue_rx: Receiver<T>,
    /// The busy worker count
    worker_idle: Arc<Counter>,
    /// The total worker count
    worker_total: Arc<Counter>,
}
impl<T, const STACK_SIZE: usize, const TIMEOUT: u64> Worker<T, STACK_SIZE, TIMEOUT> {
    /// The timeout after which an idle worker should terminate
    const TIMEOUT: Duration = Duration::from_secs(TIMEOUT);

    /// Spawns a new worker and returns it's job queue
    pub fn spawn(queue_rx: &Receiver<T>, worker_idle: &Arc<Counter>, worker_total: &Arc<Counter>) -> Result<(), Error>
    where
        T: Executable + Send + 'static,
    {
        // Create the worker
        let this =
            Self { queue_rx: queue_rx.clone(), worker_idle: worker_idle.clone(), worker_total: worker_total.clone() };

        // Spawn the thread
        let builder = Builder::new().stack_size(STACK_SIZE).name("threadpool worker thread".to_string());
        builder.spawn(|| this.runloop())?;
        Ok(())
    }

    /// The worker runloop
    fn runloop(self)
    where
        T: Executable,
    {
        // Increment the total and idle counter as long as the runloop runs
        let _worker_total_guard = self.worker_total.increment_tmp();
        let _worker_idle_guard = self.worker_idle.increment_tmp();

        // Enter the runloop
        'runloop: loop {
            // Mark use as idle and wait for the next job
            let job = match self.queue_rx.recv_timeout(Self::TIMEOUT) {
                Ok(job) => job,
                Err(_) if self.worker_total.get() < 2 => continue 'runloop,
                Err(_) => break 'runloop,
            };

            // Temporary decrement the idle counter and execute the job
            let _worker_idle_guard = self.worker_idle.decrement_tmp();
            job.exec();
        }
    }
}
