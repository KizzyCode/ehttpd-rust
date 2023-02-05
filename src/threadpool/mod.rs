//! Implements a threadpool

mod counter;
mod worker;

use crate::{
    error,
    error::Error,
    threadpool::{counter::Counter, worker::Worker},
};
use flume::{Receiver, Sender};
use std::sync::Arc;

/// A trait for functions etc. that can be executed/called, similar to `FnOnce()`
pub trait Executable {
    /// Executes `self`
    fn exec(self);
}

/// A threadpool with dynamic thread allocation and termination based on the current pressure
#[derive(Debug)]
pub struct Threadpool<T, const STACK_SIZE: usize> {
    /// The job queue to send the data to the waiting workers
    queue_tx: Sender<T>,
    /// The receiving half of the job-queue that can be passed as "seed" to newly created workers
    queue_rx_seed: Receiver<T>,
    /// The hard worker limit
    worker_max: usize,
    /// The idle worker count
    worker_idle: Arc<Counter>,
    /// The, counter::Counter} total worker count
    worker_total: Arc<Counter>,
}
impl<T, const STACK_SIZE: usize> Threadpool<T, STACK_SIZE> {
    /// Creates a new thread pool
    pub fn new(worker_max: usize) -> Self
    where
        T: Executable + Send + 'static,
    {
        // Create instance
        let (queue_tx, queue_rx_seed) = flume::bounded(worker_max);
        let worker_idle = Counter::new(0);
        let worker_total = Counter::new(0);
        Self {
            queue_tx,
            queue_rx_seed,
            worker_max,
            worker_idle: Arc::new(worker_idle),
            worker_total: Arc::new(worker_total),
        }
    }

    /// Dispatches a job into the threadpool
    pub fn dispatch(&self, job: T) -> Result<(), Error>
    where
        T: Executable + Send + 'static,
    {
        // Spawn an additional worker if we are below the baseline
        if self.worker_total.get() == 0 {
            self.spawn()?;
        }

        // Dispatch the job
        if self.queue_tx.try_send(job).is_err() {
            return Err(error!("Threadpool is congested"));
        }

        // Perform an opportunistic spawn if we have pending jobs, but ignore errors (e.g. if a resource limit is reached)
        if self.queue_tx.len() > self.worker_idle.get() {
            let _ = self.spawn();
        }
        Ok(())
    }

    /// Spawns a new worker
    fn spawn(&self) -> Result<(), Error>
    where
        T: Executable + Send + 'static,
    {
        // Check if we've reached the hard limit
        if self.worker_total.get() == self.worker_max {
            return Err(error!("Reached worker thread limit"));
        }

        // Spawn the worker
        Worker::<T, STACK_SIZE>::spawn(&self.queue_rx_seed, &self.worker_idle, &self.worker_total)
    }
}
impl<T, const STACK_SIZE: usize> Clone for Threadpool<T, STACK_SIZE> {
    fn clone(&self) -> Self {
        Self {
            queue_tx: self.queue_tx.clone(),
            queue_rx_seed: self.queue_rx_seed.clone(),
            worker_max: self.worker_max,
            worker_idle: self.worker_idle.clone(),
            worker_total: self.worker_total.clone(),
        }
    }
}
