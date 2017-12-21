#![feature(conservative_impl_trait)]

extern crate assert_cli;
extern crate hyper;
extern crate futures;
extern crate tempdir;

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
use std::fs::File;
use std::io::prelude::*;
use tempdir::TempDir;

const HOST: &str = "127.0.0.1:8060";

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
        expected_request_body: ExpectedRequestBody::JsonValue(json!([event_body])),
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
fn event_publish_command_data_update() {

    let event_body = json!({"field-2": "noooo", "field-1": 434234235});

    fn predicate(v: serde_json::Value) -> bool {
        v.as_array().and_then(|arr| {
            arr.get(0).and_then(|v| {
                v.as_object().map(|obj| {
                    obj.contains_key("metadata") &&
                        obj.get("data_op") == Some(&json!("U")) &&
                        obj.get("data").map(|data| data == &json!({"field-2": "noooo", "field-1": 434234235})).unwrap_or(false)
                })
            })
        }).unwrap_or(false)
    }

    let mocked_service = MockedService {
        body_factory: || Body::empty(),
        expected_path: "/event-types/event-type-x/events".to_string(),
        expected_request_body: ExpectedRequestBody::JsonValuePredicate(predicate),
        expected_method: Method::Post,
        status_code: StatusCode::Ok,
    };

    let shutdown = mocked_service.spawn_start(&HOST.parse().expect("Failed to parse host"));

    Assert::main_binary()
        .with_args(&["--url", &format!("http://{}", HOST), "event", "publish", "--data-update", "event-type-x", &format!("{}",event_body)])
        .succeeds()
        .execute()
        .unwrap();

    shutdown.send(()).unwrap();
}

#[test]
fn event_publish_command_data_delete() {

    let event_body = json!({"field-2": "noooo", "field-1": 434234235});

    fn predicate(v: serde_json::Value) -> bool {
        v.as_array().and_then(|arr| {
            arr.get(0).and_then(|v| {
                v.as_object().map(|obj| {
                    obj.contains_key("metadata") &&
                        obj.get("data_op") == Some(&json!("D")) &&
                        obj.get("data").map(|data| data == &json!({"field-2": "noooo", "field-1": 434234235})).unwrap_or(false)
                })
            })
        }).unwrap_or(false)
    }

    let mocked_service = MockedService {
        body_factory: || Body::empty(),
        expected_path: "/event-types/event-type-x/events".to_string(),
        expected_request_body: ExpectedRequestBody::JsonValuePredicate(predicate),
        expected_method: Method::Post,
        status_code: StatusCode::Ok,
    };

    let shutdown = mocked_service.spawn_start(&HOST.parse().expect("Failed to parse host"));

    Assert::main_binary()
        .with_args(&["--url", &format!("http://{}", HOST), "event", "publish", "--data-delete", "event-type-x", &format!("{}",event_body)])
        .succeeds()
        .execute()
        .unwrap();

    shutdown.send(()).unwrap();
}

#[test]
fn event_publish_command_data_create() {

    let event_body = json!({"field-2": "noooo", "field-1": 434234235});

    fn predicate(v: serde_json::Value) -> bool {
        v.as_array().and_then(|arr| {
            arr.get(0).and_then(|v| {
                v.as_object().map(|obj| {
                    obj.contains_key("metadata") &&
                        obj.get("data_op") == Some(&json!("C")) &&
                        obj.get("data").map(|data| data == &json!({"field-2": "noooo", "field-1": 434234235})).unwrap_or(false)
                })
            })
        }).unwrap_or(false)
    }

    let mocked_service = MockedService {
        body_factory: || Body::empty(),
        expected_path: "/event-types/event-type-x/events".to_string(),
        expected_request_body: ExpectedRequestBody::JsonValuePredicate(predicate),
        expected_method: Method::Post,
        status_code: StatusCode::Ok,
    };

    let shutdown = mocked_service.spawn_start(&HOST.parse().expect("Failed to parse host"));

    Assert::main_binary()
        .with_args(&["--url", &format!("http://{}", HOST), "event", "publish", "--data-create", "event-type-x", &format!("{}",event_body)])
        .succeeds()
        .execute()
        .unwrap();

    shutdown.send(()).unwrap();
}

#[test]
fn event_publish_command_data_snapshot() {

    let event_body = json!({"field-2": "noooo", "field-1": 434234235});

    fn predicate(v: serde_json::Value) -> bool {
        v.as_array().and_then(|arr| {
            arr.get(0).and_then(|v| {
                v.as_object().map(|obj| {
                    obj.contains_key("metadata") &&
                        obj.get("data_op") == Some(&json!("S")) &&
                        obj.get("data").map(|data| data == &json!({"field-2": "noooo", "field-1": 434234235})).unwrap_or(false)
                })
            })
        }).unwrap_or(false)
    }

    let mocked_service = MockedService {
        body_factory: || Body::empty(),
        expected_path: "/event-types/event-type-x/events".to_string(),
        expected_request_body: ExpectedRequestBody::JsonValuePredicate(predicate),
        expected_method: Method::Post,
        status_code: StatusCode::Ok,
    };

    let shutdown = mocked_service.spawn_start(&HOST.parse().expect("Failed to parse host"));

    Assert::main_binary()
        .with_args(&["--url", &format!("http://{}", HOST), "event", "publish", "--data-snapshot", "event-type-x", &format!("{}",event_body)])
        .succeeds()
        .execute()
        .unwrap();

    shutdown.send(()).unwrap();
}

#[test]
fn event_publish_command_from_file() {

    let event_body = json!({"field-2": "noooo", "field-1": 434234235});

    let dir = TempDir::new("nakacli-test").unwrap();
    let path = &format!("{}/event-body", dir.path().to_str().unwrap());

    let mut file = File::create(path).unwrap();
    file.write_all(format!("{}", event_body).as_bytes()).unwrap();

    let mocked_service = MockedService {
        body_factory: || Body::empty(),
        expected_path: "/event-types/event-type-x/events".to_string(),
        expected_request_body: ExpectedRequestBody::JsonValue(json!([event_body])),
        expected_method: Method::Post,
        status_code: StatusCode::Ok,
    };

    let shutdown = mocked_service.spawn_start(&HOST.parse().expect("Failed to parse host"));

    Assert::main_binary()
        .with_args(&["--url", &format!("http://{}", HOST), "event", "publish", "event-type-x", &format!("@{}", path)])
        .succeeds()
        .execute()
        .unwrap();

    shutdown.send(()).unwrap();
}

#[test]
fn event_publish_multiple_command() {

    let event_bodys = json!([{"field-2": "noooo", "field-1": 434234235}, {"field-2": "yes", "field-1": 6}]);

    let mocked_service = MockedService {
        body_factory: || Body::empty(),
        expected_path: "/event-types/event-type-x/events".to_string(),
        expected_request_body: ExpectedRequestBody::JsonValue(event_bodys.clone()),
        expected_method: Method::Post,
        status_code: StatusCode::Ok,
    };

    let shutdown = mocked_service.spawn_start(&HOST.parse().expect("Failed to parse host"));

    Assert::main_binary()
        .with_args(&["--url", &format!("http://{}", HOST), "event", "publish", "event-type-x", &format!("{}", event_bodys)])
        .succeeds()
        .execute()
        .unwrap();

    shutdown.send(()).unwrap();
}

#[test]
fn event_stream_command() {

    let response_body_factory = || {
        format!("{}\n{}\n{}\n",
                json!({"cursor":{"partition":"0","offset":"6"},"events":[{"field-2": "no", "field-1": 434234235}]}),
                json!({"cursor":{"partition":"0","offset":"6"},"events":[{"field-2": "noo", "field-1": 434234235}, {"field-2": "nooo", "field-1": 434234235}]}),
                json!({"cursor":{"partition":"0","offset":"6"},"events":[{"field-2": "noooo", "field-1": 434234235}]}),
        ).into()
    };

    let expected_stdout = format!("{}\n{}\n{}\n{}\n",
        json!({"field-1":434234235,"field-2":"no"}),
        json!({"field-1":434234235,"field-2":"noo"}),
        json!({"field-1":434234235,"field-2":"nooo"}),
        json!({"field-1":434234235,"field-2":"noooo"}),
    );

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

    let response_body_factory = || {
        format!("{}\n{}\n{}\n",
                json!({"cursor":{"partition":"0","offset":"6"},"events":[{"field-2": "no", "field-1": 434234235}]}),
                json!({"cursor":{"partition":"0","offset":"6"},"events":[{"field-2": "noo", "field-1": 434234235}, {"field-2": "nooo", "field-1": 434234235}]}),
                json!({"cursor":{"partition":"0","offset":"6"},"events":[{"field-2": "noooo", "field-1": 434234235}]}),
        ).into()
    };

    let expected_stdout = format!("{}\n{}\n{}\n",
        json!({"field-1":434234235,"field-2":"no"}),
        json!({"field-1":434234235,"field-2":"noo"}),
        json!({"field-1":434234235,"field-2":"nooo"}),
    );

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
    let partition_strategy = "random";
    let partition_key_fields: Option<Vec<&str>> = None;
    let compatibility_mode = "forward";

    let expected_request_body = ExpectedRequestBody::JsonValue(json!({
        "name": eventtype_name,
        "schema": {
            "type": "json_schema",
            "schema": format!("{}", eventtype_schema),
        },
        "owning_application": owning_application,
        "category": category,
        "partition_strategy": partition_strategy,
        "compatibility_mode": compatibility_mode,
        "partition_key_fields": partition_key_fields,
        "enrichment_strategies": Vec::<String>::new(),
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
        .with_args(&["--url", &format!("http://{}", HOST), "event-type", "create", owning_application, eventtype_name, &format!("{}", eventtype_schema)])
        .succeeds()
        .unwrap();

    shutdown.send(()).unwrap();
}

#[test]
fn eventtype_create_command_from_file() {

    let eventtype_schema = json!({"type":"object","properties":{"partner_id":{"type":"number"},"quantity":{"type":"number"},"app_domain":{"type":"string"},"article_id":{"type":"string"}}});

    let dir = TempDir::new("nakacli-test").unwrap();
    let path = &format!("{}/json-schema", dir.path().to_str().unwrap());

    let mut file = File::create(path).unwrap();
    file.write_all(format!("{}", eventtype_schema).as_bytes()).unwrap();

    let eventtype_name = "NEW_EVENT_TYPE";
    let owning_application = "testapp";
    let category = "undefined";
    let partition_strategy = "random";
    let partition_key_fields: Option<Vec<&str>> = None;
    let compatibility_mode = "forward";

    let expected_request_body = ExpectedRequestBody::JsonValue(json!({
        "name": eventtype_name,
        "schema": {
            "type": "json_schema",
            "schema": format!("{}", eventtype_schema),
        },
        "owning_application": owning_application,
        "category": category,
        "partition_strategy": partition_strategy,
        "compatibility_mode": compatibility_mode,
        "partition_key_fields": partition_key_fields,
        "enrichment_strategies": Vec::<String>::new(),
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
        .with_args(&["--url", &format!("http://{}", HOST), "event-type", "create", owning_application, eventtype_name, &format!("@{}", path)])
        .succeeds()
        .unwrap();

    shutdown.send(()).unwrap();
}

#[test]
fn eventtype_create_command_category_data() {

    let eventtype_schema = json!({"type":"object","properties":{"partner_id":{"type":"number"},"quantity":{"type":"number"},"app_domain":{"type":"string"},"article_id":{"type":"string"}}});
    let eventtype_name = "NEW_EVENT_TYPE";
    let owning_application = "testapp";
    let category = "data";
    let partition_strategy = "random";
    let partition_key_fields: Option<Vec<&str>> = None;
    let compatibility_mode = "forward";

    let expected_request_body = ExpectedRequestBody::JsonValue(json!({
        "name": eventtype_name,
        "schema": {
            "type": "json_schema",
            "schema": format!("{}", eventtype_schema),
        },
        "owning_application": owning_application,
        "category": category,
        "partition_strategy": partition_strategy,
        "compatibility_mode": compatibility_mode,
        "partition_key_fields": partition_key_fields,
        "enrichment_strategies": ["metadata_enrichment"],
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
        .with_args(&["--url", &format!("http://{}", HOST), "event-type", "create", "--category", category,  owning_application, eventtype_name, &format!("{}", eventtype_schema)])
        .succeeds()
        .unwrap();

    shutdown.send(()).unwrap();
}

#[test]
fn eventtype_create_command_category_business() {

    let eventtype_schema = json!({"type":"object","properties":{"partner_id":{"type":"number"},"quantity":{"type":"number"},"app_domain":{"type":"string"},"article_id":{"type":"string"}}});
    let eventtype_name = "NEW_EVENT_TYPE";
    let owning_application = "testapp";
    let category = "business";
    let partition_strategy = "random";
    let partition_key_fields: Option<Vec<&str>> = None;
    let compatibility_mode = "forward";

    let expected_request_body = ExpectedRequestBody::JsonValue(json!({
        "name": eventtype_name,
        "schema": {
            "type": "json_schema",
            "schema": format!("{}", eventtype_schema),
        },
        "owning_application": owning_application,
        "category": category,
        "partition_strategy": partition_strategy,
        "compatibility_mode": compatibility_mode,
        "partition_key_fields": partition_key_fields,
        "enrichment_strategies": ["metadata_enrichment"],
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
        .with_args(&["--url", &format!("http://{}", HOST), "event-type", "create", "--category", category,  owning_application, eventtype_name, &format!("{}", eventtype_schema)])
        .succeeds()
        .unwrap();

    shutdown.send(()).unwrap();
}

#[test]
fn eventtype_delete_command() {

    let eventtype_name = "NEW_EVENT_TYPE";

    let mocked_service = MockedService {
        body_factory: || Body::empty(),
        expected_path: format!("/event-types/{}", eventtype_name),
        expected_request_body: ExpectedRequestBody::None,
        expected_method: Method::Delete,
        status_code: StatusCode::Ok,
    };

    let shutdown = mocked_service.spawn_start(&HOST.parse().expect("Failed to parse host"));

    Assert::main_binary()
        .with_args(&["--url", &format!("http://{}", HOST), "event-type", "delete", eventtype_name])
        .succeeds()
        .unwrap();

    shutdown.send(()).unwrap();
}

#[test]
fn eventtype_list_command() {

    let list_response = json!([
        {
            "name": "event1",
            "owning_application": "app1",
            "category": "business",
            "enrichment_strategies": [ "metadata_enrichment" ],
            "partition_strategy": "hash",
            "partition_key_fields": [ "field1" ],
            "default_statistic": { "messages_per_minute": 100, "message_size": 100, "read_parallelism": 8, "write_parallelism": 8 },
            "options": { "retention_time": 345600000 },
            "authorization": null,
            "compatibility_mode": "forward",
            "updated_at": "2017-06-19T13:11:24.943Z",
            "created_at": "2017-06-19T13:11:24.943Z"
        },
        {
            "name": "event2",
            "owning_application": "app2",
            "category": "business",
            "enrichment_strategies": [ "metadata_enrichment" ],
            "partition_strategy": "random",
            "partition_key_fields": [],
            "schema": { "type": "json_schema", "schema": "{ \"properties\": { \"json\": { \"type\": \"string\" } }}", "version": "1.0.0", "created_at": "2017-10-16T09:47:42.408Z" },
            "default_statistic": null,
            "options": { "retention_time": 345600000 },
            "authorization": null,
            "compatibility_mode": "forward",
            "updated_at": "2017-10-16T09:47:42.408Z",
            "created_at": "2017-10-16T09:47:42.408Z"
        }]);

    let bf = || {
        format!("{}", json!([
        {
            "name": "event1",
            "owning_application": "app1",
            "category": "business",
            "enrichment_strategies": [ "metadata_enrichment" ],
            "partition_strategy": "hash",
            "partition_key_fields": [ "field1" ],
            "default_statistic": { "messages_per_minute": 100, "message_size": 100, "read_parallelism": 8, "write_parallelism": 8 },
            "options": { "retention_time": 345600000 },
            "authorization": null,
            "compatibility_mode": "forward",
            "updated_at": "2017-06-19T13:11:24.943Z",
            "created_at": "2017-06-19T13:11:24.943Z"
        },
        {
            "name": "event2",
            "owning_application": "app2",
            "category": "business",
            "enrichment_strategies": [ "metadata_enrichment" ],
            "partition_strategy": "random",
            "partition_key_fields": [],
            "schema": { "type": "json_schema", "schema": "{ \"properties\": { \"json\": { \"type\": \"string\" } }}", "version": "1.0.0", "created_at": "2017-10-16T09:47:42.408Z" },
            "default_statistic": null,
            "options": { "retention_time": 345600000 },
            "authorization": null,
            "compatibility_mode": "forward",
            "updated_at": "2017-10-16T09:47:42.408Z",
            "created_at": "2017-10-16T09:47:42.408Z"
        }])).into()
    };

    let mocked_service = MockedService {
        body_factory: bf,
        expected_path: "/event-types".to_string(),
        expected_request_body: ExpectedRequestBody::None,
        expected_method: Method::Get,
        status_code: StatusCode::Ok,
    };

    let shutdown = mocked_service.spawn_start(&HOST.parse().expect("Failed to parse host"));

    Assert::main_binary()
        .with_args(&["--url", &format!("http://{}", HOST), "event-type", "list"])
        .stdout().is(format!("{}", list_response))
        .succeeds()
        .execute()
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
    JsonValuePredicate(fn(Value) -> bool),
//    Text(String),
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
                    ExpectedRequestBody::JsonValuePredicate(p) if Some(true) == serde_json::from_str(&request_body).ok().map(p) => good_response,
//                    ExpectedRequestBody::Text(ref expected_request_text) if expected_request_text == &request_body => good_response,
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
