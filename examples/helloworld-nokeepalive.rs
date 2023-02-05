use ehttpd::{
    bytes::Source,
    http::{Request, Response, ResponseExt},
    Server,
};

fn main() {
    // Define our request handler
    let request_handler = |_: Request| {
        let mut response = Response::new_200_ok();
        response.set_body(Source::from(b"Hello World")).expect("failed to get set body");
        response.set_connection_close();
        response
    };

    // Create a server that listens at [::]:9999 with up to 2048 worker threads under load if necessary
    let server: Server = Server::new(2048, request_handler);
    server.accept("[::]:9999").expect("server failed");
}
