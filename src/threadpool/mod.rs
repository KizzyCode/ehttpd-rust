//! Implements a threadpool

mod worker;

use crate::{error, error::Error, threadpool::worker::Worker};
use flume::{Receiver, Sender};
use std::{
    panic::UnwindSafe,
    sync::{
        atomic::{AtomicUsize, Ordering::SeqCst},
        Arc,
    },
};

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
    /// The total worker count
    workers: Arc<AtomicUsize>,
}
impl<T, const STACK_SIZE: usize> Threadpool<T, STACK_SIZE> {
    /// Creates a new thread pool
    pub fn new(worker_max: usize) -> Self
    where
        T: Executable + UnwindSafe + Send + 'static,
    {
        // Create queues and counter
        let (queue_tx, queue_rx_seed) = flume::bounded(worker_max);
        let workers = Arc::new(AtomicUsize::default());
        Self { queue_tx, queue_rx_seed, workers }
    }

    /// Dispatches a job into the threadpool
    pub fn dispatch(&self, job: T) -> Result<(), Error>
    where
        T: Executable + Send + UnwindSafe + 'static,
    {
        // Spawn workers as necessary
        let worker_count = self.workers.load(SeqCst);
        if worker_count == 0 {
            // We need at least one worker, so required spawn
            self.spawn()?;
        }
        if worker_count <= self.queue_tx.len() {
            // More workers would be better, so opportunistic spawn
            let _ = self.spawn();
        }

        // Dispatch the job
        self.queue_tx.try_send(job).map_err(|_| error!("Threadpool is congested"))?;
        Ok(())
    }

    /// Spawns a new worker
    fn spawn(&self) -> Result<(), Error>
    where
        T: Executable + Send + UnwindSafe + 'static,
    {
        // Check if we've reached the hard limit
        if Some(self.workers.load(SeqCst)) >= self.queue_tx.capacity() {
            return Err(error!("Worker limit exceeded"));
        }

        // Spawn the worker
        Worker::<T, STACK_SIZE>::spawn(self.queue_rx_seed.clone(), self.workers.clone())
    }
}
impl<T, const STACK_SIZE: usize> Clone for Threadpool<T, STACK_SIZE> {
    fn clone(&self) -> Self {
        Self {
            queue_tx: self.queue_tx.clone(),
            queue_rx_seed: self.queue_rx_seed.clone(),
            workers: self.workers.clone(),
        }
    }
}
