#![doc = include_str!("../README.md")]

pub mod error;
pub mod http;
pub mod log;
pub mod threadpool;
pub mod utils;

use crate::{
    error::Error,
    http::{Request, Response},
    threadpool::{Pool, StaticFn},
};
use std::{
    convert::Infallible,
    io::BufReader,
    net::{Shutdown, TcpListener, TcpStream},
    sync::{
        mpsc::{self, Receiver, SyncSender},
        Arc,
    },
    thread::Builder,
};

/// A connection context to pass to the thread pool
struct ConnectionContext {
    /// The connection handler
    #[allow(clippy::type_complexity)]
    pub handler: Arc<Box<dyn Fn(&mut Request) -> Response + Send + Sync + 'static>>,
    /// The TCP stream to handle
    pub stream: BufReader<TcpStream>,
    /// The connection queue for keep-alice TCP connections
    pub connection_queue: SyncSender<BufReader<TcpStream>>,
}

/// A HTTP server
pub struct Server<const STACK_SIZE: usize = 65_536> {
    /// The underlying socket
    socket: TcpListener,
    /// The thread pool to handle the incoming connections
    threadpool: Pool<ConnectionContext, STACK_SIZE>,
    /// The connection queue "seed" for pending TCP connections
    connection_queue_seed: SyncSender<BufReader<TcpStream>>,
    /// The keep-alive queue for pending TCP connections
    connection_queue: Receiver<BufReader<TcpStream>>,
}
impl<const STACK_SIZE: usize> Server<STACK_SIZE> {
    /// Creates a new server bound on the given address
    pub fn new(address: &str, soft_limit: usize, hard_limit: usize) -> Result<Self, Error> {
        // Bind the socket and create threadpool and queues
        let socket = TcpListener::bind(address)?;
        let threadpool: Pool<_, STACK_SIZE> = Pool::new(soft_limit, hard_limit);
        let (connection_queue_seed, connection_queue) = mpsc::sync_channel(hard_limit);

        Ok(Self { socket, threadpool, connection_queue_seed, connection_queue })
    }

    /// Starts the server
    pub fn exec<F>(self, callback: F) -> Result<Infallible, Error>
    where
        F: Fn(&mut Request) -> Response + Send + Sync + 'static,
    {
        // Box the given callback to distribute it across worker threads
        let callback = {
            let boxed: Box<dyn Fn(&mut Request) -> Response + Send + Sync + 'static> = Box::new(callback);
            Arc::new(boxed)
        };

        // Start the acceptor thread and process incoming connections
        Self::accept_async(self.socket, &self.connection_queue_seed)?;
        loop {
            // Receive the next connection and create the context
            let connection = self.connection_queue.recv().expect("connection queue is broken");
            let context = ConnectionContext {
                handler: callback.clone(),
                stream: connection,
                connection_queue: self.connection_queue_seed.clone(),
            };

            // Schedule the connection for processing
            let job = StaticFn { fn_: Self::callback_executor, context };
            self.threadpool.schedule(job)?;
        }
    }

    /// Spawns an `accept` thread that inserts new connections into the
    fn accept_async(socket: TcpListener, connection_queue: &SyncSender<BufReader<TcpStream>>) -> Result<(), Error> {
        // Duplicate the queue
        let connection_queue = connection_queue.clone();

        // Start the acceptor thread
        let builder = Builder::new().name("acceptor thread".to_string()).stack_size(STACK_SIZE);
        builder.spawn(move || loop {
            // Accept a connection
            if let Ok((connection, _)) = socket.accept() {
                let connection = BufReader::new(connection);
                connection_queue.send(connection).expect("cannot schedule connection for processing")
            }
        })?;
        Ok(())
    }

    /// Calls a callback with the associated connection
    fn callback_executor(context: ConnectionContext) {
        /// Tries to call a callback with the associated connection
        fn try_(context: ConnectionContext) -> Result<(), Error> {
            // Destructure the context and read the request
            let ConnectionContext { handler, stream, connection_queue } = context;
            let Some(mut request) = Request::from_stream(stream)? else {
                // The stream has been closed immediately, due to keep-alive this is not necessarily an error
                return Ok(());
            };

            // Create the response and reacquire the stream
            let mut response: Response = handler(&mut request);
            let mut stream = request.stream;

            // Write the response and reschedule the connection if it is still alive
            response.to_stream(stream.get_mut())?;
            match response.has_connection_close() {
                true => stream.get_ref().shutdown(Shutdown::Both)?,
                false => connection_queue.send(stream).expect("cannot reschedule connection for processing"),
            }
            Ok(())
        }

        // Handle the connection or log an error if necessary
        if let Err(e) = try_(context) {
            // Build the error string
            let mut error_string = e.to_string();
            if e.has_backtrace() {
                writeln!(&mut error_string, "{}", e.backtrace).expect("failed to format backtrace");
            }

            // Log the error
            log_info!("Failed to handle connection: {error_string}");
        }
    }
}
