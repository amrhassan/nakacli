
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

pub const NAME:             &str = "publish";
const ARG_EVENT_TYPE:       &str = "event-type";
const ARG_JSON_BODY:        &str = "json-body";
const ARG_ENRICH_METADATA:  &str = "enrich-metadata";

pub fn sub_command<'a>() -> App<'a, 'a> {
    SubCommand::with_name(NAME)
        .about("Publish one or more events")
        .arg(Arg::with_name(ARG_EVENT_TYPE).required(true).index(1).help("Name of the Event Type"))
        .arg(Arg::with_name(ARG_ENRICH_METADATA).required(false).takes_value(false).long("enrich-metadata").short("m").help("Enriches required metadata fields for data/business events if missing"))
        .arg(Arg::with_name(ARG_JSON_BODY)
            .required(true)
            .index(2)
            .help("Body of one or more events in JSON format (Use '@' prefix to specify a filepath. e.g. '@event.json')")
            .validator(validate_json_body)
        )
}

struct Params<'a> {
    event_type: &'a str,
    json_body: String,
    enrich_metadata: bool,
}

fn extract_params<'a>(matches: &'a ArgMatches) -> Params<'a> {
    Params {
        event_type: matches.value_of(ARG_EVENT_TYPE).expect("Non-optional argument should have been caught by clap if missing"),
        json_body: matches.value_of(ARG_JSON_BODY).and_then(|v| long_argument(v).ok()).expect("Non-optional argument should have been caught by clap if missing"),
        enrich_metadata: matches.occurrences_of(ARG_ENRICH_METADATA) > 0,
    }
}

pub fn run(application: &mut Application, global_params: &GlobalParams, matches: &ArgMatches) {
    let params = extract_params(matches);
    let server_info = ServerInfo::from_params(global_params);
    let path = format!("/event-types/{}/events", params.event_type);

    let decoded_events = serde_json::from_str::<serde_json::Value>(&params.json_body).expect("Failed to JSON-decode text that was validated to be JSON by clap");

    let body: serde_json::Value = match request_body(decoded_events, params.enrich_metadata) {
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

fn request_body(events: serde_json::Value, enrich_metadata: bool) -> Result<serde_json::Value, Failure> {

    let events_maybe: Result<Vec<serde_json::Map<String, serde_json::Value>>, Failure> =
        match events {
            serde_json::Value::Array(event_values) => {
                event_values.into_iter()
                    .map(|v| match v { serde_json::Value::Object(obj) => Ok(obj), _ => Err(failure("Provided event array contains a non-object")) })
                    .collect()
            },
            serde_json::Value::Object(ev) => Ok(vec![ev]),
            _ => Err(failure("Provided input must either be a JSON object or an array objects")),
        };

    let mut events: Vec<serde_json::Map<String, serde_json::Value>> = events_maybe?;

    if enrich_metadata {
        for event in events.iter_mut() {
            match enrich_with_metadata(event) {
                Ok(_)       => (),
                Err(err)    => return Err(err)
            }
        }
    }

    Ok(serde_json::Value::Array(events.into_iter().map(serde_json::Value::Object).collect()))
}

/// Enriches an event or an array of events
fn enrich_with_metadata(event: &mut serde_json::Map<String, serde_json::Value>) -> Result<(), Failure> {
    match event.entry("metadata").or_insert(json!({})).as_object_mut() {
      Some(metadata) => {
          metadata.entry("eid").or_insert(json!(format!("{}", Uuid::new_v4())));
          metadata.entry("occurred_at").or_insert(json!(format!("{}", Local::now())));
          Ok(())
      },
      _ => Err(failure("Provided event already contains a metadata field but it's not an object!"))
  }
}

fn validate_json_body(value: String) -> Result<(), String> {
    match serde_json::from_str::<serde_json::Value>(&long_argument(&value)?) {
        Err(err) => Err(format!("JSON body of event is malformed: {}", err)),
        Ok(json_value) =>
            if let Some(array) = json_value.as_array() {
                if array.iter().all(|obj| obj.is_object()) {
                    Ok(())
                } else {
                    Err(format!("JSON body should be an array of objects if you're trying to publish multiple events"))
                }
            } else if !json_value.is_object() {
                Err("JSON body of event needs to be an object".to_owned())
            } else {
                Ok(())
            }
    }
}

