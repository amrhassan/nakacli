#![feature(conservative_impl_trait)]

extern crate assert_cli;
extern crate hyper;
extern crate futures;

#[macro_use]
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
use hyper::Body;
use std::net::SocketAddr;

const HOST: &'static str = "127.0.0.1:8060";

#[test]
fn metrics_command() {

    let metrics_response = json!({"metrics": "all is good"});

    let mocked_service = MockedService {
        body_factory: || format!("{}", json!({"metrics": "all is good"})).into(),
        expected_path: "/metrics".to_string(),
        expected_request_body: ExpectedRequestBody::None,
        expected_method: Method::Get,
        status_code: StatusCode::Ok,
    };

    let shutdown = mocked_service.spawn_start(&HOST.parse().expect("Failed to parse host"));

    Assert::main_binary()
        .with_args(&["--url", &format!("http://{}", HOST), "metrics"])
        .stdout().is(format!("{}", metrics_response))
        .succeeds()
        .execute()
        .unwrap();

    shutdown.send(()).unwrap();
}

#[test]
fn event_publish_command() {

    let event_body = json!({"field-2": "noooo", "field-1": 434234235});

    let mocked_service = MockedService {
        body_factory: || Body::empty(),
        expected_path: "/event-types/event-type-x/events".to_string(),
        expected_request_body: ExpectedRequestBody::JsonValue(json!([{"field-2": "noooo", "field-1": 434234235}])),
        expected_method: Method::Post,
        status_code: StatusCode::Ok,
    };

    let shutdown = mocked_service.spawn_start(&HOST.parse().expect("Failed to parse host"));

    Assert::main_binary()
        .with_args(&["--url", &format!("http://{}", HOST), "event", "publish", "event-type-x", &format!("{}",event_body)])
        .succeeds()
        .execute()
        .unwrap();

    shutdown.send(()).unwrap();
}

#[test]
fn event_publish_multiple_command() {

    let event_bodys = "[{\"field-2\": \"noooo\", \"field-1\": 434234235}, {\"field-2\": \"yes\", \"field-1\": 6}]";

    let mocked_service = MockedService {
        body_factory: || Body::empty(),
        expected_path: "/event-types/event-type-x/events".to_string(),
        expected_request_body: ExpectedRequestBody::JsonValue(serde_json::from_str("[{\"field-2\": \"noooo\", \"field-1\": 434234235}, {\"field-2\": \"yes\", \"field-1\": 6}]").expect("BAD JSON")),
        expected_method: Method::Post,
        status_code: StatusCode::Ok,
    };

    let shutdown = mocked_service.spawn_start(&HOST.parse().expect("Failed to parse host"));

    Assert::main_binary()
        .with_args(&["--url", &format!("http://{}", HOST), "event", "publish", "event-type-x", event_bodys])
        .succeeds()
        .execute()
        .unwrap();

    shutdown.send(()).unwrap();
}

#[test]
fn event_stream_command() {

    let response_body_factory = || "\
    {\"cursor\":{\"partition\":\"0\",\"offset\":\"6\"},\"events\":[{\"field-2\": \"no\", \"field-1\": 434234235}]}\n\
    {\"cursor\":{\"partition\":\"0\",\"offset\":\"6\"},\"events\":[{\"field-2\": \"noo\", \"field-1\": 434234235}, {\"field-2\": \"nooo\", \"field-1\": 434234235}]}\n\
    {\"cursor\":{\"partition\":\"0\",\"offset\":\"6\"},\"events\":[{\"field-2\": \"noooo\", \"field-1\": 434234235}]}\n\
    ".to_string().into();

    let expected_stdout = "\
    {\"field-1\":434234235,\"field-2\":\"no\"}\n\
    {\"field-1\":434234235,\"field-2\":\"noo\"}\n\
    {\"field-1\":434234235,\"field-2\":\"nooo\"}\n\
    {\"field-1\":434234235,\"field-2\":\"noooo\"}\n\
    ";

    let mocked_service = MockedService {
        body_factory: response_body_factory,
        expected_path: "/event-types/event-type-x/events".to_string(),
        expected_request_body: ExpectedRequestBody::None,
        expected_method: Method::Get,
        status_code: StatusCode::Ok,
    };


    let shutdown = mocked_service.spawn_start(&HOST.parse().expect("Failed to parse host"));

    Assert::main_binary()
        .with_args(&["--url", &format!("http://{}", HOST), "event", "stream", "event-type-x"])
        .stdout().is(expected_stdout)
        .fails()    // Because stream ends abruptly. TODO: Fix with a hanging body stream
        .unwrap();

    shutdown.send(()).unwrap();
}

#[test]
fn event_stream_n_command() {

    let response_body_factory = || "\
    {\"cursor\":{\"partition\":\"0\",\"offset\":\"6\"},\"events\":[{\"field-2\": \"no\", \"field-1\": 434234235}]}\n\
    {\"cursor\":{\"partition\":\"0\",\"offset\":\"6\"},\"events\":[{\"field-2\": \"noo\", \"field-1\": 434234235}, {\"field-2\": \"nooo\", \"field-1\": 434234235}]}\n\
    {\"cursor\":{\"partition\":\"0\",\"offset\":\"6\"},\"events\":[{\"field-2\": \"noooo\", \"field-1\": 434234235}]}\n\
    ".to_string().into();

    let expected_stdout = "\
    {\"field-1\":434234235,\"field-2\":\"no\"}\n\
    {\"field-1\":434234235,\"field-2\":\"noo\"}\n\
    {\"field-1\":434234235,\"field-2\":\"nooo\"}\n\
    ";

    let mocked_service = MockedService {
        body_factory: response_body_factory,
        expected_path: "/event-types/event-type-x/events".to_string(),
        expected_request_body: ExpectedRequestBody::None,
        expected_method: Method::Get,
        status_code: StatusCode::Ok,
    };


    let shutdown = mocked_service.spawn_start(&HOST.parse().expect("Failed to parse host"));

    Assert::main_binary()
        .with_args(&["--url", &format!("http://{}", HOST), "event", "stream", "-n3", "event-type-x"])
        .stdout().is(expected_stdout)
        .unwrap();

    shutdown.send(()).unwrap();
}

#[test]
fn eventtype_create_command() {

    let eventtype_schema = json!({"type":"object","properties":{"partner_id":{"type":"number"},"quantity":{"type":"number"},"app_domain":{"type":"string"},"article_id":{"type":"string"}}});
    let eventtype_name = "NEW_EVENT_TYPE";
    let owning_application = "testapp";
    let category = "undefined";

    let expected_request_body = ExpectedRequestBody::JsonValue(json!({
        "name": eventtype_name,
        "schema": {
            "type": "json_schema",
            "schema": serde_json::to_string(&eventtype_schema).expect("Failed to encode JSON Schema of event type into String"),
        },
        "owning_application": owning_application,
        "category": category,
        }));

    let mocked_service = MockedService {
        body_factory: || Body::empty(),
        expected_path: "/event-types".to_string(),
        expected_request_body,
        expected_method: Method::Post,
        status_code: StatusCode::Created,
    };

    let shutdown = mocked_service.spawn_start(&HOST.parse().expect("Failed to parse host"));

    Assert::main_binary()
        .with_args(&["--url", &format!("http://{}", HOST), "event-type", "create", owning_application, eventtype_name, &serde_json::to_string(&eventtype_schema).unwrap()])
        .succeeds()
        .unwrap();

    shutdown.send(()).unwrap();
}

#[derive(Clone, Debug)]
struct MockedService {
    body_factory: fn() -> Body, // Not a closure because needs to be cloneable. Maybe after https://git.io/vF747 this can be done
    expected_path: String,
    expected_request_body: ExpectedRequestBody,
    expected_method: Method,
    status_code: StatusCode,
}

#[derive(Clone, Debug)]
enum ExpectedRequestBody {
    JsonValue(Value),
    Text(String),
    None
}

impl Service for MockedService {

    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error=Self::Error> + 'static>;

    fn call(&self, req: Self::Request) -> Self::Future {
        let mocked_service = Clone::clone(self);
        if req.method() == &mocked_service.expected_method && req.path() == &mocked_service.expected_path {
            Box::new(req.body().concat2().map(move |chunk| {
                let request_body_bytes: Vec<u8> = chunk.into_iter().collect();
                let request_body = String::from_utf8(request_body_bytes).expect("Failed to UTF-8 decode request body");
                let body_factory = mocked_service.body_factory;

                let good_response = Response::new().with_status(mocked_service.status_code.clone()).with_body(body_factory());

                match mocked_service.expected_request_body {
                    ExpectedRequestBody::JsonValue(ref expected_request_json_value) if Some(expected_request_json_value) == serde_json::from_str(&request_body).ok().as_ref() => good_response,
                    ExpectedRequestBody::Text(ref expected_request_text) if expected_request_text == &request_body => good_response,
                    ExpectedRequestBody::None => good_response,
                    _ => {
                        eprintln!("Unexpected request body: {} vs {:?}", &request_body, mocked_service.expected_request_body);
                        Response::new().with_status(StatusCode::BadRequest)
                    }
                }
            }))
        } else {
            eprintln!("Unexpected request: {} {}", &mocked_service.expected_method, &mocked_service.expected_path);
            Box::new(future::ok(Response::new().with_status(StatusCode::NotFound)))
        }
    }
}

impl MockedService {

    /// Spawns a web server in a new thread. Returns a Sender that can be used to shutdown the server.
    fn spawn_start(self, host: &SocketAddr) -> Sender<()> {
        let address = host.clone();
        let (tx, rx) = channel();
        thread::spawn(move || {
            Http::new().bind(&address, move || Ok(self.clone()))
                .expect("Failed to start HTTP server")
                .run_until(rx.map_err(|err| panic!(err)))
                .expect("HTTP server got interrupted")
        });
        tx
    }
}
