#[cfg(feature = "server")]
fn main() {
    use ehttpd::{
        Server,
        http::{Response, ResponseExt},
    };

    // Create a server that listens at [::]:9999 with up to 2048 worker threads under load if necessary
    let server = Server::with_request_response(2048, |_| {
        let mut response = Response::new_200_ok();
        response.set_body_data(b"Hello world\r\n");
        response
    });

    // Handle incoming connections
    server.accept("[::]:9999").expect("server failed");
}

#[cfg(not(feature = "server"))]
fn main() {
    panic!("The `server`-feature must be enabled for this example to run")
}
