use ehttpd::{
    bytes::{Sink, Source},
    http::{Request, Response, ResponseExt},
    Server,
};

fn main() {
    // Define our request handler
    let connection_handler = |source: &mut Source, sink: &mut Sink| {
        // Handle request
        ehttpd::reqresp(source, sink, |_: Request| {
            let mut response = Response::new_200_ok();
            response.set_body_data(b"Hello world\r\n");
            response.set_connection_close();
            response
        })
    };

    // Create a server that listens at [::]:9999 with up to 2048 worker threads under load if necessary
    let server: Server<_> = Server::new(2048, connection_handler);
    server.accept("[::]:9999").expect("server failed");
}
