//! Implements a threadpool

mod counter;
mod worker;

use crate::{
    error,
    error::Error,
    threadpool::{counter::Counter, worker::Worker},
};
use flume::{Receiver, Sender};
use std::{panic::UnwindSafe, sync::Arc};

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
    /// The receiving half of the `queue_tx` job-queue that can be passed as "seed" to newly created workers
    queue_rx_seed: Receiver<T>,
    /// The idle worker count
    workers_idle: Arc<Counter>,
    /// The total worker count
    workers_total: Arc<Counter>,
}
impl<T, const STACK_SIZE: usize> Threadpool<T, STACK_SIZE> {
    /// Creates a new thread pool
    pub fn new(worker_max: usize) -> Self
    where
        T: Executable + UnwindSafe + Send + 'static,
    {
        // Create queues and counter
        let (queue_tx, queue_rx_seed) = flume::bounded(worker_max);
        let workers_idle = Arc::new(Counter::new(0));
        let workers_total = Arc::new(Counter::new(0));
        Self { queue_tx, queue_rx_seed, workers_idle, workers_total }
    }

    /// Dispatches a job into the threadpool
    pub fn dispatch(&self, job: T) -> Result<(), Error>
    where
        T: Executable + Send + UnwindSafe + 'static,
    {
        // Spawn an additional worker if there is no running worker left (e.g. due to a panic)
        if self.workers_total.get() == 0 {
            self.spawn()?;
        }

        // Dispatch the job
        if self.queue_tx.try_send(job).is_err() {
            return Err(error!("Threadpool is congested"));
        }

        // Perform an opportunistic spawn if we have pending jobs, but ignore errors (e.g. if a resource limit is reached)
        if self.queue_tx.len() > self.workers_idle.get() {
            let _ = self.spawn();
        }
        Ok(())
    }

    /// Spawns a new worker
    fn spawn(&self) -> Result<(), Error>
    where
        T: Executable + Send + UnwindSafe + 'static,
    {
        // Check if we've reached the hard limit
        if Some(self.workers_total.get()) >= self.queue_tx.capacity() {
            return Err(error!("Reached worker limit"));
        }

        // Spawn the worker
        Worker::<T, STACK_SIZE, 5>::spawn(&self.queue_rx_seed, &self.workers_idle, &self.workers_total)
    }
}
impl<T, const STACK_SIZE: usize> Clone for Threadpool<T, STACK_SIZE> {
    fn clone(&self) -> Self {
        Self {
            queue_tx: self.queue_tx.clone(),
            queue_rx_seed: self.queue_rx_seed.clone(),
            workers_idle: self.workers_idle.clone(),
            workers_total: self.workers_total.clone(),
        }
    }
}
