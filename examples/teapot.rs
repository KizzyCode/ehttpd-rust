#[cfg(feature = "server")]
fn main() {
    use ehttpd::{
        http::{Response, ResponseExt},
        Server,
    };

    // Create a server that listens at [::]:9999 with up to 2048 worker threads under load if necessary
    let server = Server::with_request_response(2048, |request| {
        // Create the response body
        let mut message = b"There are only teapots in ".to_vec();
        message.extend_from_slice(&request.target);
        message.extend_from_slice(b"\r\n");

        // Send the response
        let mut response = Response::new_status_reason(418, "I'm a teapot");
        response.set_body_data(message);
        response
    });

    // Handle incoming connections
    server.accept("[::]:9999").expect("server failed");
}

#[cfg(not(feature = "server"))]
fn main() {
    panic!("The `server`-feature must be enabled for this example to run")
}
