
use server::ServerInfo;
use app::Application;
use serde_json;
use http;
use hyper::{Method, StatusCode};
use output;
use global::*;
use clap::{ArgMatches, App, SubCommand, Arg};
use input::long_argument;

pub const NAME: &str = "publish";
const ARG_EVENT_TYPE: &str = "event-type";
const ARG_JSON_BODY: &str = "json-body";

pub fn sub_command<'a>() -> App<'a, 'a> {
    SubCommand::with_name(NAME)
        .about("Publish one or more events")
        .arg(Arg::with_name(ARG_EVENT_TYPE).required(true).index(1).help("Name of the Event Type"))
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
}

fn extract_params<'a>(matches: &'a ArgMatches) -> Params<'a> {
    Params {
        event_type: matches.value_of(ARG_EVENT_TYPE).expect("Non-optional argument should have been caught by clap if missing"),
        json_body: matches.value_of(ARG_JSON_BODY).and_then(|v| long_argument(v).ok()).expect("Non-optional argument should have been caught by clap if missing")
    }
}

pub fn run(application: &mut Application, global_params: &GlobalParams, matches: &ArgMatches) {
    let params = extract_params(matches);
    let server_info = ServerInfo::from_params(global_params);
    let path = format!("/event-types/{}/events", params.event_type);
    let body = {
        let decoded = serde_json::from_str::<serde_json::Value>(&params.json_body).expect("Failed to JSON-decode text that was validated to be JSON by clap");
        if decoded.is_object() {
            serde_json::Value::Array(vec![decoded])
        } else if decoded.is_array() {
            decoded
        } else {
            panic!("Input json_body should have been validated to be JSON object or array by clap")
        }
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

