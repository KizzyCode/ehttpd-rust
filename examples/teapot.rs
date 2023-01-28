use ehttpd::{
    http::{
        request::Request,
        response::Response,
        responseext::{ResponseBodyExt, ResponseExt},
    },
    log::{self, WARN},
    Server,
};

fn main() {
    // Log everything
    log::set_level(WARN);

    // Define our request handler
    let request_handler = |request: &mut Request| {
        // Create the response body
        let mut message = b"There are only teapots in ".to_vec();
        message.extend_from_slice(&request.target);
        message.extend_from_slice(b"\r\n");

        // Send the response
        let mut response = Response::new_status_reason(418, "I'm a teapot");
        response.set_body_data(message);
        response
    };

    // Create a server that listens at [::]:9999, keeps up to 64 worker threads *permanently* and can spawn up to 1024
    // temporary worker threads under high load
    let server: Server = Server::new("[::]:9999", 64, 4096).expect("failed to start server");
    server.exec(request_handler).expect("server failed");
}
