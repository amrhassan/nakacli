extern crate assert_cli;
extern crate tokio_service;
extern crate hyper;
extern crate futures;

use hyper::server::{Http, Request, Response, Service};
use hyper::{Method, StatusCode};
use assert_cli::{Assert};
use futures::Future;
use futures::future;
use std::thread::*;

#[test]
fn metrics_command() {

    let port = 8080;
    let host = format!("127.0.0.1:{}", port);
    let address = host.parse().unwrap();

    spawn(move || {
        Http::new().bind(&address, || Ok(MockNakadi))
            .unwrap()
            .run()
            .unwrap();
    });

    Assert::main_binary()
        .with_args(&["--url", &format!("http://{}", host), "metrics"])
        .stdout().is("{\"metrics\": \"all is good\"}")
        .unwrap();
}

struct MockNakadi;

impl Service for MockNakadi {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        let mut response = Response::new();
        match (req.method(), req.path()) {
            (&Method::Get, "/metrics") => {
                response.set_status(StatusCode::Ok);
                response.set_body("{\"metrics\": \"all is good\"}");
            },
            _ => response.set_status(StatusCode::InternalServerError)
        }
        Box::new(future::ok(response))
    }
}
