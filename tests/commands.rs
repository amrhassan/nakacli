#![feature(conservative_impl_trait)]

extern crate assert_cli;
extern crate hyper;
extern crate futures;
extern crate serde_json;

use hyper::server::{Http, Request, Response, Service};
use hyper::{Method, StatusCode};
use assert_cli::Assert;
use futures::Future;
use futures::Stream;
use futures::future;
use std::thread;
use futures::sync::oneshot::{channel, Sender};
use serde_json::Value;
use std::time::Duration;

#[test]
fn metrics_command() {
    let metrics_response = "{\"metrics\": \"all is good\"}";

    let shutdown = spawn_mock_nakadi(Method::Get, "/metrics".to_string(), None, metrics_response.to_string(), StatusCode::Ok);

    Assert::main_binary()
        .with_args(&["--url", &format!("http://{}", MockNakadi::HOST), "metrics"])
        .stdout().is(metrics_response)
        .succeeds()
        .execute()
        .unwrap();

    shutdown.send(()).unwrap();
}

#[test]
fn event_publish_command() {
    let response_body = "";
    let event_body = "{\"field-2\": \"noooo\", \"field-1\": 434234235}";
    let expected_request_body = serde_json::from_str("[{\"field-2\": \"noooo\", \"field-1\": 434234235}]").expect("BAD JSON");

    let shutdown = spawn_mock_nakadi(Method::Post, "/event-types/event-type-x/events".to_owned(), Some(expected_request_body), response_body.to_owned(), StatusCode::Ok);

    Assert::main_binary()
        .with_args(&["--url", &format!("http://{}", MockNakadi::HOST), "event", "publish", "event-type-x", event_body])
        .succeeds()
        .execute()
        .unwrap();

    shutdown.send(()).unwrap();
}

#[test]
fn event_stream_command() {
    let response_body = "\
    {\"cursor\":{\"partition\":\"0\",\"offset\":\"6\"},\"events\":[{\"field-2\": \"no\", \"field-1\": 434234235}]}\n\
    {\"cursor\":{\"partition\":\"0\",\"offset\":\"6\"},\"events\":[{\"field-2\": \"noo\", \"field-1\": 434234235}, {\"field-2\": \"nooo\", \"field-1\": 434234235}]}\n\
    {\"cursor\":{\"partition\":\"0\",\"offset\":\"6\"},\"events\":[{\"field-2\": \"noooo\", \"field-1\": 434234235}]}\n\
    ";
    let expected_stdout = "\
    {\"field-1\":434234235,\"field-2\":\"no\"}\n\
    {\"field-1\":434234235,\"field-2\":\"noo\"}\n\
    {\"field-1\":434234235,\"field-2\":\"nooo\"}\n\
    {\"field-1\":434234235,\"field-2\":\"noooo\"}\n\
    ";

    let event_name = "event-type-x";

    let shutdown = spawn_mock_nakadi(Method::Get, format!("/event-types/{}/events", event_name), None, response_body.to_owned(), StatusCode::Ok);

    thread::sleep(Duration::from_secs(24*60*60));

    Assert::main_binary()
        .with_args(&["--url", &format!("http://{}", MockNakadi::HOST), "event", "stream", event_name])
        .stdout().is(expected_stdout)
        .fails()
        .unwrap();

    shutdown.send(()).unwrap();
}

#[test]
fn event_stream_n_command() {
    let response_body = "\
    {\"cursor\":{\"partition\":\"0\",\"offset\":\"6\"},\"events\":[{\"field-2\": \"no\", \"field-1\": 434234235}]}\n\
    {\"cursor\":{\"partition\":\"0\",\"offset\":\"6\"},\"events\":[{\"field-2\": \"noo\", \"field-1\": 434234235}, {\"field-2\": \"nooo\", \"field-1\": 434234235}]}\n\
    {\"cursor\":{\"partition\":\"0\",\"offset\":\"6\"},\"events\":[{\"field-2\": \"noooo\", \"field-1\": 434234235}]}\n\
    ";
    let expected_stdout = "\
    {\"field-1\":434234235,\"field-2\":\"no\"}\n\
    {\"field-1\":434234235,\"field-2\":\"noo\"}\n\
    ";

    let event_name = "event-type-x";

    let shutdown = spawn_mock_nakadi(Method::Get, format!("/event-types/{}/events", event_name), None, response_body.to_owned(), StatusCode::Ok);

    Assert::main_binary()
        .with_args(&["--url", &format!("http://{}", MockNakadi::HOST), "event", "stream", "-n2", event_name])
        .stdout().is(expected_stdout)
        .succeeds()
        .unwrap();

    shutdown.send(()).unwrap();
}

#[test]
fn event_publish_multiple() {
    // TODO
}

fn spawn_mock_nakadi(
    expected_method: Method,
    expected_path: String,
    expected_request_body: Option<Value>,
    response_body: String,
    status_code: StatusCode) -> Sender<()> {
    let address = MockNakadi::HOST.parse().unwrap();
    let (tx, rx) = channel();
    thread::spawn(move || {
        Http::new().bind(&address, move || Ok(MockNakadi { expected_method: expected_method.clone(), expected_path: expected_path.clone(), response_body: response_body.clone(), status_code: status_code.clone(), expected_request_body: expected_request_body.clone() }))
            .unwrap()
            .run_until(rx.map_err(|err| panic!(err)))
            .unwrap()
    });
    tx
}

struct MockNakadi {
    expected_method: Method,
    expected_request_body: Option<Value>,
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
                if serde_json::from_str(&request_body).ok() == expected_request_body {
                    Response::new().with_status(status_code).with_body(response_body)
                } else {
                    println!("ERROR: Request body: {:?} does not equal expected body: {:?}", request_body, expected_request_body);
                    Response::new().with_status(StatusCode::InternalServerError)
                }
            }))
        } else {
            println!("Unexpected request: {} {}", self.expected_method, self.expected_path);
            Box::new(future::ok(Response::new().with_status(StatusCode::InternalServerError)))
        }
    }
}
