//! Implements a threadpool

mod worker;

use crate::{error, error::Error, log_error, log_info, threadpool::worker::Worker};
use std::sync::{
    mpsc::{self, Receiver, SyncSender},
    Arc,
};

/// A function pointer together with a context argument
///
/// This struct serves as a leightweight and limited replacement for boxed closures.
#[derive(Debug, Clone, Copy)]
pub struct StaticFn<T> {
    /// The function to call
    pub fn_: fn(T),
    /// The context to pass to the function
    pub context: T,
}

/// A threadpool
pub struct Pool<T, const STACK_SIZE: usize> {
    /// The waiting workers
    workers: Receiver<SyncSender<StaticFn<T>>>,
    /// The "seed" of the `workers`-queue to pass it to new workers
    workers_seed: SyncSender<SyncSender<StaticFn<T>>>,
    /// The worker counter
    worker_counter: Arc<()>,
    /// The hard thread limit
    hard_limit: usize,
}
impl<T, const STACK_SIZE: usize> Pool<T, STACK_SIZE> {
    /// Creates a new thread pool
    pub fn new(soft_limit: usize, hard_limit: usize) -> Self {
        let (workers_seed, workers) = mpsc::sync_channel(soft_limit);
        Self { workers, workers_seed, worker_counter: Arc::new(()), hard_limit }
    }

    /// Schedules a job
    pub fn schedule(&self, job: StaticFn<T>) -> Result<(), Error>
    where
        T: Send + 'static,
    {
        // Get a free worker or spawn a new one if necessary
        let worker = match self.workers.try_recv() {
            Ok(worker) => worker,
            Err(_) => self.spawn_worker()?,
        };

        // Send the job to the worker
        worker.try_send(job).expect("available worker is unable to accept thread");
        Ok(())
    }

    /// Spawns a new worker
    fn spawn_worker(&self) -> Result<SyncSender<StaticFn<T>>, Error>
    where
        T: Send + 'static,
    {
        // Check if we've reached the hard limit
        if Arc::strong_count(&self.worker_counter) == self.hard_limit {
            log_error!("Cannot spawn new worker because we have reached the hard limit");
            return Err(error!("Thread pool is saturated"));
        }

        // Spawn the worker
        log_info!("Spawning new worker");
        Worker::<T, STACK_SIZE>::spawn(&self.workers_seed, &self.worker_counter)
    }
}
