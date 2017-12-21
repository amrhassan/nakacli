
use server::ServerInfo;
use app::Application;
use serde_json;
use http;
use hyper::{Method, StatusCode};
use output;
use global::*;
use clap::{ArgMatches, App, SubCommand, Arg};
use input::long_argument;
use uuid::Uuid;
use chrono::prelude::*;
use output::{Failure, failure, die_failure};
use arg_validators;
use std::fmt;

pub const NAME:             &str = "publish";
const ARG_EVENT_TYPE:       &str = "event-type";
const ARG_JSON_BODY:        &str = "json-body";
const ARG_DATA_CREATE:      &str = "data-create";
const ARG_DATA_UPDATE:      &str = "data-update";
const ARG_DATA_DELETE:      &str = "data-delete";
const ARG_DATA_SNAPSHOT:    &str = "data-snapshot";
const ARG_BUSINESS:         &str = "business";

pub fn sub_command<'a>() -> App<'a, 'a> {
    SubCommand::with_name(NAME)
        .about("Publish one or more events")
        .arg(Arg::with_name(ARG_EVENT_TYPE).required(true).index(1).help("Name of the Event Type"))
        .arg(Arg::with_name(ARG_DATA_CREATE)
            .required(false)
            .takes_value(false)
            .long("data-create")
            .help("Publish as a data event with creation op")
            .conflicts_with(ARG_DATA_DELETE)
            .conflicts_with(ARG_DATA_SNAPSHOT)
            .conflicts_with(ARG_DATA_UPDATE)
            .conflicts_with(ARG_BUSINESS))
        .arg(Arg::with_name(ARG_DATA_UPDATE)
           .required(false)
           .takes_value(false)
           .long("data-update")
           .help("Publish as a data event with update op")
           .conflicts_with(ARG_DATA_DELETE)
           .conflicts_with(ARG_DATA_SNAPSHOT)
           .conflicts_with(ARG_DATA_CREATE)
           .conflicts_with(ARG_BUSINESS))
        .arg(Arg::with_name(ARG_DATA_DELETE)
           .required(false)
           .takes_value(false)
           .long("data-delete")
           .help("Publish as a data event with deletion op")
           .conflicts_with(ARG_DATA_UPDATE)
           .conflicts_with(ARG_DATA_SNAPSHOT)
           .conflicts_with(ARG_DATA_CREATE)
           .conflicts_with(ARG_BUSINESS))
        .arg(Arg::with_name(ARG_DATA_SNAPSHOT)
           .required(false)
           .takes_value(false)
           .long("data-snapshot")
           .help("Publish as a data event with snapshot op")
           .conflicts_with(ARG_DATA_UPDATE)
           .conflicts_with(ARG_DATA_DELETE)
           .conflicts_with(ARG_DATA_CREATE)
           .conflicts_with(ARG_BUSINESS))
        .arg(Arg::with_name(ARG_BUSINESS)
            .required(false)
           .takes_value(false)
           .long("business")
           .help("Publish as a business event")
           .conflicts_with(ARG_DATA_UPDATE)
           .conflicts_with(ARG_DATA_SNAPSHOT)
           .conflicts_with(ARG_DATA_CREATE)
           .conflicts_with(ARG_DATA_DELETE))
        .arg(Arg::with_name(ARG_JSON_BODY)
            .required(true)
            .index(2)
            .help("Body of one or more events in JSON format (Use '@' prefix to specify a filepath. e.g. '@event.json')")
            .validator(arg_validators::json)
        )
}

enum Category {
    Undefined,
    Data { op: DataOp },
    Business,
}

#[derive(Copy, Clone)]
enum DataOp {
    Create,
    Update,
    Deletion,
    Snapshot,
}

impl fmt::Display for DataOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            &DataOp::Create => "C",
            &DataOp::Update => "U",
            &DataOp::Deletion => "D",
            &DataOp::Snapshot => "S",
        })
    }
}

struct Params {
    event_type: EventType,
    json_body: serde_json::Value,
    category: Category,
}

struct EventType(String);

fn decode_params(matches: &ArgMatches) -> Params {

    let category =
        if matches.occurrences_of(ARG_DATA_UPDATE) > 0 {
            Category::Data { op: DataOp::Update }
        } else if matches.occurrences_of(ARG_DATA_CREATE) > 0 {
            Category::Data { op: DataOp::Create }
        } else if matches.occurrences_of(ARG_DATA_DELETE) > 0 {
            Category::Data { op: DataOp::Deletion }
        } else if matches.occurrences_of(ARG_DATA_SNAPSHOT) > 0 {
            Category::Data { op: DataOp::Snapshot }
        } else if matches.occurrences_of(ARG_BUSINESS) > 0 {
            Category::Business
        } else {
            Category::Undefined
        };

    let json_body_str = matches
        .value_of(ARG_JSON_BODY)
        .and_then(|v| long_argument(v).ok())
        .expect("Non-optional argument should have been caught by clap if missing");

    let json_body =
        serde_json::from_str::<serde_json::Value>(&json_body_str).expect("Failed to JSON-decode text that was validated to be JSON by clap");

    let event_type = EventType(matches.value_of(ARG_EVENT_TYPE).expect("Non-optional argument should have been caught by clap if missing").to_owned());

    Params { event_type, json_body, category }
}

pub fn run(application: &mut Application, global_params: &GlobalParams, matches: &ArgMatches) {
    let Params { event_type, json_body, category } = decode_params(matches);
    let server_info = ServerInfo::from_params(global_params);
    let path = format!("/event-types/{}/events", event_type.0);

    let body_maybe = match category {
        Category::Undefined => request_for_undefined(json_body),
        Category::Data { op } => request_for_data(&event_type, json_body, op),
        _ => unimplemented!()
    };

    let body: serde_json::Value = match body_maybe {
        Ok(b)       => b,
        Err(err)    => die_failure(err),    // weird, can't do .unwrap_or_else(die_failure) because ! is not a proper bottom type?
    };

    let action = http::execute_and_read_full_resp_body_utf8(
        &application.http_client,
        Method::Post,
        &path,
        &server_info,
        Some(&body)
    );
    let result = application.core.run(action);
    output::final_result(result, StatusCode::Ok, global_params.pretty)
}

fn request_for_undefined(json_body: serde_json::Value) -> Result<serde_json::Value, Failure> {
    match json_body {
        arr@serde_json::Value::Array(_) => Ok(arr),
        obj@serde_json::Value::Object(_)  => Ok(serde_json::Value::Array(vec![obj])),
        _ => Err(failure("Event must be either a JSON object or an array of JSON objects."))
    }
}

fn request_for_data(event_type: &EventType ,json_body: serde_json::Value, op: DataOp) -> Result<serde_json::Value, Failure> {

    let data_event = |event: serde_json::Value| -> Result<serde_json::Value, Failure> {
        let now = Local::now().to_rfc3339();
        match event {
            obj@serde_json::Value::Object(_) =>
                Ok(json!({
                    "data": obj,
                    "data_op": format!("{}", op),
                    "data_type": event_type.0,
                    "metadata": {
                        "eid": format!("{}", Uuid::new_v4()),
                        "occurred_at": now,
                    }
                })),
            _ => Err(failure("Provided JSON must be an array of objects"))
        }
    };

    match json_body {
        serde_json::Value::Array(objs) => {
            let events: Result<Vec<serde_json::Value>, Failure> = objs.into_iter().map(data_event).collect();
            events.map(serde_json::Value::Array)
        },
        obj@serde_json::Value::Object(_)  => data_event(obj).map(|event| serde_json::Value::Array(vec![event])),
        _ => Err(failure("Event must be either a JSON object or an array of JSON objects."))
    }
}

