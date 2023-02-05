//! An atomic counter with increment- and decrement-guards

use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};

/// A counter-operation guard which undoes the operation if it goes out of scope
pub struct CounterOpGuard<'a> {
    /// The guarded counter
    counter: &'a Counter,
    /// The operation to perform on drop
    on_drop: fn(&'a Counter),
}
impl<'a> Drop for CounterOpGuard<'a> {
    fn drop(&mut self) {
        (self.on_drop)(self.counter);
    }
}

/// An atomic `usize` counter with increment- and decrement-guards
#[derive(Debug)]
pub struct Counter {
    /// The counter value
    counter: AtomicUsize,
}
impl Counter {
    /// Creates a new counter
    pub fn new(value: usize) -> Self {
        Self { counter: AtomicUsize::new(value) }
    }

    /// Gets the current counter value
    pub fn get(&self) -> usize {
        self.counter.load(SeqCst)
    }

    /// Increments the counter by one
    pub fn increment(&self) {
        self.counter.fetch_add(1, SeqCst);
    }
    /// Decrements the counter by one
    pub fn decrement(&self) {
        self.counter.fetch_sub(1, SeqCst);
    }

    /// Performs a temporary increment of the counter by one; the operation is undone if the returned guard is dropped
    pub fn increment_tmp(&self) -> CounterOpGuard {
        self.increment();
        CounterOpGuard { counter: self, on_drop: Self::decrement }
    }
    /// Performs a temporary decrement of the counter by one; the operation is undone if the returned guard is dropped
    pub fn decrement_tmp(&self) -> CounterOpGuard {
        self.decrement();
        CounterOpGuard { counter: self, on_drop: Self::increment }
    }
}
