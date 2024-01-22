#![doc = include_str!("../README.md")]

pub mod bytes;
pub mod error;
pub mod http;
pub mod threadpool;

use crate::{
    bytes::{Sink, Source},
    error::Error,
    http::{Request, Response},
    threadpool::{Executable, Threadpool},
};
use std::{
    convert::Infallible,
    io::BufReader,
    net::{TcpListener, ToSocketAddrs},
    panic::UnwindSafe,
    sync::Arc,
};

/// A connection to pass to the thread pool
struct Connection<T, const STACK_SIZE: usize> {
    /// The connection handler
    pub handler: T,
    /// The receiving half of the stream
    pub rx: Source,
    /// The writing half of the stream
    pub tx: Sink,
    /// The connection queue for keep-alice TCP connections
    pub threadpool: Arc<Threadpool<Self, STACK_SIZE>>,
}
impl<T, const STACK_SIZE: usize> Connection<T, STACK_SIZE>
where
    T: Fn(&mut Source, &mut Sink) -> bool + Send + Sync + UnwindSafe + 'static,
{
    /// Handles the connection
    fn handle(mut self) -> Result<(), Error> {
        // Call the connection handler
        if (self.handler)(&mut self.rx, &mut self.tx) {
            // Reschedule the connection
            let threadpool = self.threadpool.clone();
            threadpool.dispatch(self)?;
        }
        Ok(())
    }
}
impl<T, const STACK_SIZE: usize> Executable for Connection<T, STACK_SIZE>
where
    T: Fn(&mut Source, &mut Sink) -> bool + Send + Sync + UnwindSafe + 'static,
{
    fn exec(self) {
        let _ = self.handle();
    }
}

/// A HTTP server
pub struct Server<T, const STACK_SIZE: usize = 65_536> {
    /// The thread pool to handle the incoming connections
    threadpool: Arc<Threadpool<Connection<T, STACK_SIZE>, STACK_SIZE>>,
    /// The connection handler
    handler: T,
}
impl<T, const STACK_SIZE: usize> Server<T, STACK_SIZE>
where
    T: Fn(&mut Source, &mut Sink) -> bool + Clone + Send + Sync + UnwindSafe + 'static,
{
    /// Creates a new server bound on the given address
    pub fn new(worker_max: usize, handler: T) -> Self {
        // Create threadpool and init self
        let threadpool: Threadpool<_, STACK_SIZE> = Threadpool::new(worker_max);
        Self { threadpool: Arc::new(threadpool), handler }
    }

    /// Dispatches a connection
    pub fn dispatch(&self, rx: Source, tx: Sink) -> Result<(), Error> {
        // Create and dispatch the job
        let job = Connection { handler: self.handler.clone(), rx, tx, threadpool: self.threadpool.clone() };
        self.threadpool.dispatch(job)
    }

    /// Listens on the given address and accepts forever
    pub fn accept<A>(self, address: A) -> Result<Infallible, Error>
    where
        A: ToSocketAddrs,
    {
        // Bind and listen
        let socket = TcpListener::bind(address)?;
        loop {
            // Accept and prepare connection
            let (stream, _) = socket.accept()?;
            let tx = stream.try_clone()?;
            let rx = BufReader::new(stream);

            // Dispatch connection
            let rx = Source::from_other(rx);
            self.dispatch(rx, tx.into())?;
        }
    }
}

/// An adapter to bridge a `source,sink`-handler to a `request->response`-handler
#[must_use]
pub fn reqresp<F>(source: &mut Source, sink: &mut Sink, handler: F) -> bool
where
    F: Fn(Request) -> Response + Send + Sync + UnwindSafe + 'static,
{
    // Read request
    let Ok(Some(request)) = Request::from_stream(source) else {
        return false;
    };

    // Handle request and write response
    let mut response = handler(request);
    let Ok(_) = response.to_stream(sink) else {
        return false;
    };

    // Mark connection as to-be-rescheduled
    !response.has_connection_close()
}
