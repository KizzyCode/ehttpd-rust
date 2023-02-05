use ehttpd::{
    http::{Request, Response, ResponseExt},
    Server,
};

fn main() {
    // Define our request handler
    let request_handler = |request: Request| {
        // Create the response body
        let mut message = b"There are only teapots in ".to_vec();
        message.extend_from_slice(&request.target);
        message.extend_from_slice(b"\r\n");

        // Send the response
        let mut response = Response::new_status_reason(418, "I'm a teapot");
        response.set_body_data(message);
        response
    };

    // Create a server that listens at [::]:9999 with up to 2048 worker threads under load if necessary
    let server: Server = Server::new(2048, request_handler);
    server.accept("[::]:9999").expect("server failed");
}
