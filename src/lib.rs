#![doc = include_str!("../README.md")]

pub mod bytes;
pub mod error;
pub mod http;
pub mod threadpool;

use crate::{
    bytes::Source,
    error::Error,
    http::{Request, Response},
    threadpool::{Executable, Threadpool},
};
use std::{
    convert::Infallible,
    io::BufReader,
    net::{Shutdown, TcpListener, TcpStream, ToSocketAddrs},
    sync::Arc,
};

/// A connection handler job to pass to the thread pool
struct ConnectionJob<const STACK_SIZE: usize> {
    /// The connection handler
    pub handler: Arc<dyn Fn(Request) -> Response + Send + Sync + 'static>,
    /// The receiving half of the stream
    pub rx: Source,
    /// The writing half of the stream
    pub tx: TcpStream,
    /// The connection queue for keep-alice TCP connections
    pub threadpool: Arc<Threadpool<Self, STACK_SIZE>>,
}
impl<const STACK_SIZE: usize> ConnectionJob<STACK_SIZE> {
    /// Handles the connection
    fn handle(mut self) -> Result<(), Error> {
        // Read the request
        let Some(request) = Request::from_stream(&mut self.rx)? else {
            // The stream has been closed immediately – due to keep-alive this is not necessarily an error
            return Ok(());
        };

        // Create the response and reacquire the stream
        let mut response = (self.handler)(request);
        response.to_stream(&mut self.tx)?;

        // Close the connection if it is not kept-alive
        if response.has_connection_close() {
            self.tx.shutdown(Shutdown::Both)?;
            return Ok(());
        }

        // Reschedule the connection
        let threadpool = self.threadpool.clone();
        threadpool.dispatch(self)?;
        Ok(())
    }
}
impl<const STACK_SIZE: usize> Executable for ConnectionJob<STACK_SIZE> {
    fn exec(self) {
        let _ = self.handle();
    }
}

/// A HTTP server
pub struct Server<const STACK_SIZE: usize = 65_536> {
    /// The thread pool to handle the incoming connections
    threadpool: Arc<Threadpool<ConnectionJob<STACK_SIZE>, STACK_SIZE>>,
    /// The connection handler
    handler: Arc<dyn Fn(Request) -> Response + Send + Sync + 'static>,
}
impl<const STACK_SIZE: usize> Server<STACK_SIZE> {
    /// Creates a new server bound on the given address
    pub fn new<T>(worker_max: usize, handler: T) -> Self
    where
        T: Fn(Request) -> Response + Send + Sync + 'static,
    {
        // Create threadpool and
        let threadpool: Threadpool<_, STACK_SIZE> = Threadpool::new(worker_max);
        Self { threadpool: Arc::new(threadpool), handler: Arc::new(handler) }
    }

    /// Dispatches an incoming TCP stream
    pub fn dispatch(&self, connection: TcpStream) -> Result<(), Error> {
        // Split the connection into RX and TX
        let tx = connection.try_clone()?;
        let rx = Source::from_other(BufReader::new(connection));

        // Create and dispatch the job
        let job = ConnectionJob { handler: self.handler.clone(), rx, tx, threadpool: self.threadpool.clone() };
        self.threadpool.dispatch(job)
    }

    /// Listens on the given address and accepts forever
    pub fn accept<T>(self, address: T) -> Result<Infallible, Error>
    where
        T: ToSocketAddrs,
    {
        // Bind and listen
        let socket = TcpListener::bind(address)?;
        loop {
            // Accept and dispatch
            let (stream, _) = socket.accept()?;
            self.dispatch(stream)?;
        }
    }
}
