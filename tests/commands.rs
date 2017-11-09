#![feature(conservative_impl_trait)]

extern crate assert_cli;
extern crate hyper;
extern crate futures;

use hyper::server::{Http, Request, Response, Service};
use hyper::{Method, StatusCode};
use assert_cli::{Assert};
use futures::Future;
use futures::Stream;
use futures::future;
use std::thread;
use futures::sync::oneshot::{channel, Sender};

#[test]
fn metrics_command() {

    let metrics_response = "{\"metrics\": \"all is good\"}";

    let shutdown = spawn_mock_nakadi(Method::Get, "/metrics".to_string(), "".to_string(), metrics_response.to_string(), StatusCode::Ok);

    Assert::main_binary()
        .with_args(&["--url", &format!("http://{}", MockNakadi::HOST), "metrics"])
        .stdout().is(metrics_response)
        .unwrap();

    shutdown.send(()).unwrap();
}

#[test]
fn event_publish_command() {

    let response_body = "";
    let event_body = "{\"field-2\": \"noooo\"}";
    let expected_request_body = "[{\"field-2\":\"noooo\"}]";

    let shutdown = spawn_mock_nakadi(Method::Post, "/event-types/event-type-x/events".to_owned(), expected_request_body.to_owned(), response_body.to_owned(), StatusCode::Ok);

    Assert::main_binary()
        .with_args(&["--url", &format!("http://{}", MockNakadi::HOST), "event", "publish", "event-type-x", event_body])
        .unwrap();

    shutdown.send(()).unwrap();
}

fn spawn_mock_nakadi(
    expected_method: Method,
    expected_path: String,
    expected_request_body: String,
    response_body: String,
    status_code: StatusCode) -> Sender<()> {
    let address = MockNakadi::HOST.parse().unwrap();
    let (tx, rx) = channel();
    thread::spawn(move || {
        Http::new().bind(&address, move || Ok(MockNakadi {expected_method: expected_method.clone(), expected_path: expected_path.clone(), response_body: response_body.clone(), status_code: status_code.clone(), expected_request_body: expected_request_body.clone()}))
            .unwrap()
            .run_until(rx.map_err(|err| panic!(err)))
            .unwrap()
    });
    tx
}

struct MockNakadi {
    expected_method: Method,
    expected_request_body: String,
    expected_path: String,
    response_body: String,
    status_code: StatusCode,
}

impl MockNakadi {
    const HOST: &'static str = "127.0.0.1:8080";
}

impl Service for MockNakadi {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        if req.method() == &self.expected_method && req.path() == &self.expected_path {
            let expected_request_body = self.expected_request_body.clone();
            let response_body = self.response_body.clone();
            let status_code = self.status_code.clone();

            Box::new(req.body().concat2().map(move |chunk| {
                let request_body_bytes: Vec<u8> = chunk.into_iter().collect();
                let request_body = String::from_utf8(request_body_bytes).expect("Failed to UTF-8 decode request body");
                if request_body == expected_request_body {  // TODO: Maybe be more robust about checking JSON bodies for equality to avoid inequality due to different formatting
                    Response::new().with_status(status_code).with_body(response_body)
                } else {
                    println!("ERROR: Request body: {} does not equal expected body: {}", request_body, expected_request_body);
                    Response::new().with_status(StatusCode::InternalServerError)
                }
            }))
        } else {
            Box::new(future::ok(Response::new().with_status(StatusCode::InternalServerError)))
        }
    }
}
