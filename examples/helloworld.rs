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
    let request_handler = |_: &mut Request| {
        let mut response = Response::new_200_ok();
        response.set_body_static(b"Hello world\r\n");
        response
    };

    // Create a server that listens at [::]:9999, keeps up to 64 worker threads *permanently* and can spawn up to 1024
    // temporary worker threads under high load
    let server: Server = Server::new("[::]:9999", 64, 4096).expect("failed to start server");
    server.exec(request_handler).expect("server failed");
}
