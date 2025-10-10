//! Implements a simple threadpool-based server
#![cfg(feature = "server")]

mod pool;
mod worker;

use crate::{
    bytes::{Sink, Source},
    error::Error,
    http::{Request, Response},
    server::pool::{Executable, Threadpool},
};
use std::{
    convert::Infallible,
    io::{BufReader, BufWriter, Write},
    net::{TcpListener, ToSocketAddrs},
    sync::Arc,
};

/// A connection handler
#[derive(Clone)]
enum Handler {
    /// A `source,sink`-handler
    SourceSink(Arc<dyn Fn(&mut Source, &mut Sink) -> bool + Send + Sync + 'static>),
    /// A `request->response`-handler
    RequestResponse(Arc<dyn Fn(Request) -> Response + Send + Sync + 'static>),
}
impl Handler {
    /// Handles a given connection and returns whether the handler wants to be rescheduled (e.g. keep-alive)
    pub fn exec(&self, source: &mut Source, sink: &mut Sink) -> bool {
        match self {
            Handler::SourceSink(handler) => handler(source, sink),
            Handler::RequestResponse(handler) => Self::bridge_request_response(source, sink, handler.as_ref()),
        }
    }

    /// Bridges a `request->response`-handler to a source-sink pattern
    fn bridge_request_response<F>(source: &mut Source, sink: &mut Sink, handler: &F) -> bool
    where
        F: Fn(Request) -> Response + ?Sized,
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
}

/// An encapsulated connection to pass to the thread pool
struct Connection<const STACK_SIZE: usize> {
    /// The connection handler
    pub handler: Handler,
    /// The receiving half of the stream
    pub source: Source,
    /// The writing half of the stream
    pub sink: Sink,
    /// The thread-pool so that keep-alive connections can requeue themselves
    pub threadpool: Threadpool<Self, STACK_SIZE>,
}
impl<const STACK_SIZE: usize> Executable for Connection<STACK_SIZE> {
    fn exec(mut self) {
        // Call the connection handler and flush the output
        let reschedule = self.handler.exec(&mut self.source, &mut self.sink);
        if self.sink.flush().is_ok() && reschedule {
            // Reschedule the connection
            let threadpool = self.threadpool.clone();
            let _ = threadpool.dispatch(self);
        }
    }
}

/// A threadpool-based HTTP server
pub struct Server<const STACK_SIZE: usize> {
    /// The thread pool to handle the incoming connections
    threadpool: Threadpool<Connection<STACK_SIZE>, STACK_SIZE>,
    /// The connection handler
    handler: Handler,
}
impl<const STACK_SIZE: usize> Server<STACK_SIZE> {
    /// Creates a new server with the given connection handler
    pub fn with_source_sink<F>(workers_max: usize, source_sink_handler: F) -> Self
    where
        F: Fn(&mut Source, &mut Sink) -> bool + Send + Sync + 'static,
    {
        // Create threadpool and init self
        let threadpool: Threadpool<_, STACK_SIZE> = Threadpool::new(workers_max);
        let handler = Handler::SourceSink(Arc::new(source_sink_handler));
        Self { threadpool, handler }
    }
    /// Creates a new server with the given connection handler
    pub fn with_request_response<F>(workers_max: usize, request_response_handler: F) -> Self
    where
        F: Fn(Request) -> Response + Send + Sync + 'static,
    {
        // Create threadpool and init self
        let threadpool: Threadpool<_, STACK_SIZE> = Threadpool::new(workers_max);
        let handler = Handler::RequestResponse(Arc::new(request_response_handler));
        Self { threadpool, handler }
    }

    /// Manually dispatches a connection
    pub fn dispatch(&self, source: Source, sink: Sink) -> Result<(), Error> {
        // Create and dispatch the job
        self.threadpool.dispatch(Connection {
            handler: self.handler.clone(),
            source,
            sink,
            threadpool: self.threadpool.clone(),
        })
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
            let (source, _) = socket.accept()?;
            let sink = source.try_clone()?;

            // Dispatch connection
            let source = BufReader::new(source);
            let sink = BufWriter::new(sink);
            self.dispatch(source.into(), sink.into())?;
        }
    }
}
