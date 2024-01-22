//! A thread worker

use crate::{error::Error, threadpool::Executable};
use flume::Receiver;
use std::{
    panic::{self, UnwindSafe},
    sync::{
        atomic::{AtomicUsize, Ordering::SeqCst},
        Arc,
    },
    thread::Builder,
    time::{Duration, Instant},
};

/// A thread
pub struct Worker<T, const STACK_SIZE: usize> {
    /// The receiving half of the job-queue
    queue_rx: Receiver<T>,
    /// The total worker count
    worker: Arc<AtomicUsize>,
}
impl<T, const STACK_SIZE: usize> Worker<T, STACK_SIZE> {
    /// Timeout after which workers consider themselves idle or dispatch operations timeout
    const TIMEOUT: Duration = Duration::from_secs(4);
    /// The 1/N chance for a worker to terminate if idle
    const TERMCHANCE: u128 = 8;

    /// Spawns a new worker and returns it's job queue
    pub fn spawn(queue_rx: Receiver<T>, worker: Arc<AtomicUsize>) -> Result<(), Error>
    where
        T: Executable + Send + UnwindSafe + 'static,
    {
        // Create the worker and increment counter
        worker.fetch_add(1, SeqCst);
        let this = Self { queue_rx, worker };

        // Spawn the thread
        let builder = Builder::new().stack_size(STACK_SIZE).name("threadpool worker thread".to_string());
        builder.spawn(|| this.runloop())?;
        Ok(())
    }

    /// The worker runloop
    fn runloop(self)
    where
        T: Executable + UnwindSafe,
    {
        'runloop: loop {
            // Mark use as idle and wait for the next job
            let Ok(job) = self.queue_rx.recv_timeout(Self::TIMEOUT) else {
                // Roll whether to continue or terminate
                match Instant::now().elapsed().as_nanos() % Self::TERMCHANCE {
                    0 => break 'runloop,
                    _ => continue 'runloop,
                }
            };

            // Execute job
            let _ = panic::catch_unwind(|| job.exec());
        }
    }
}
impl<T, const STACK_SIZE: usize> Drop for Worker<T, STACK_SIZE> {
    fn drop(&mut self) {
        self.worker.fetch_sub(1, SeqCst);
    }
}
