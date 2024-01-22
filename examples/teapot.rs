use ehttpd::{
    bytes::{Sink, Source},
    http::{Response, ResponseExt},
    Server,
};

fn main() {
    // Define our request handler
    let connection_handler = |source: &mut Source, sink: &mut Sink| {
        // Handle request
        ehttpd::reqresp(source, sink, |request| {
            // Create the response body
            let mut message = b"There are only teapots in ".to_vec();
            message.extend_from_slice(&request.target);
            message.extend_from_slice(b"\r\n");

            // Send the response
            let mut response = Response::new_status_reason(418, "I'm a teapot");
            response.set_body_data(message);
            response
        })
    };

    // Create a server that listens at [::]:9999 with up to 2048 worker threads under load if necessary
    let server: Server<_> = Server::new(2048, connection_handler);
    server.accept("[::]:9999").expect("server failed");
}
