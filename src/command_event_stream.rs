use http::*;
use futures::future;
use server::ServerInfo;
use futures::Future;
use app::Application;
use hyper::{Method, Response, StatusCode};
use output::{die, failure, print_json_value, Failure, die_successfully};
use futures::Stream;
use serde_json::{Value, Map, from_str};
use global::*;
use clap::{ArgMatches, App, SubCommand, Arg};
use futures::stream;

pub const NAME: &'static str = "stream";
const ARG_EVENT_TYPE: &'static str = "event-type";
const ARG_TAKE: &'static str = "take";

struct Params<'a> {
    event_type: &'a str,
    take: Option<usize>,
}

fn extract_params<'a>(matches: &'a ArgMatches) -> Params<'a> {
    Params {
        event_type: matches.value_of(ARG_EVENT_TYPE).expect("Non-optional argument should have been caught by clap if missing"),
        take: matches.value_of(ARG_TAKE).and_then(|v| v.parse().ok()),
    }
}

pub fn sub_command<'a>() -> App<'a, 'a> {
    SubCommand::with_name(NAME)
        .about("Stream-listen on published events")
        .arg(Arg::with_name(ARG_EVENT_TYPE).required(true).index(1).help("Name of the Event Type"))
        .arg(Arg::with_name(ARG_TAKE).long("take").short("n").takes_value(true).value_name("N").help("Exits after consuming N events from the stream").validator(arg_take_validator))
}

pub fn run(application: &mut Application, global_params: &GlobalParams, matches: &ArgMatches) {
    let params = extract_params(matches);
    let server_info = ServerInfo::from_params(global_params);

    let path = format!("/event-types/{}/events", params.event_type);
    let method = Method::Get;
    let body = None;
    let request = build_request(method, &path, &server_info, body);
    let http_client = &application.http_client;

    let action = future::result(request)
        .and_then(move |r| execute_request(http_client, r))
        .and_then(|resp| process_response(resp, global_params, &params));

    match application.core.run(action) {
        Err(err) => die(1, err),
        Ok(_) => die(1, failure("Stream ended abrputly!", ""))
    }
}

fn process_response<'a>(resp: Response, global_params: &'a GlobalParams<'a>, params: &'a Params<'a>) -> impl Future<Item=(Vec<u8>, usize), Error=Failure> + 'a {
    if resp.status() != StatusCode::Ok {
        die(1, failure("Unexpected status code", resp.status()))
    } else {
        resp.body()
            .map(|chunk| {
                let bytes: Vec<u8> = chunk.into_iter().collect();
                stream::iter_ok(bytes)
            })
            .map_err(|err| failure("Failed to stream HTTP chunks", err))
            .flatten()
            .fold((Vec::new(), 0), move |(acc, i), byte| {

                if byte == b'\n' {
                    let line = String::from_utf8(acc).expect("Failed to UTF-8 decode the response");

                    let batch: EventBatch = {
                        match from_str(&line) {
                            Err(err) => die(1, failure("Failed to decode an event stream batch", err)),
                            Ok(batch) => batch
                        }
                    };

                    if let Some(events) = batch.events {
                        let event_length = events.len();
                        if let Some(take_n) = params.take {
                            for event in events.into_iter().take(take_n) {
                                print_json_value(&Value::Object(event), global_params.clone().pretty)
                            }
                            if (i+event_length) >= take_n {
                                die_successfully();
                            }
                        }
                        future::ok((Vec::new(), i+event_length))
                    } else {
                        future::ok((Vec::new(), i))
                    }
                } else {
                    let mut mut_acc = acc;
                    mut_acc.push(byte);
                    future::ok((mut_acc, i))
                }
            })
    }
}

fn arg_take_validator(v: String) -> Result<(), String> {
    match v.parse::<u64>() {
        Ok(n) if n > 0 => Ok(()),
        _ => Err("Value should be a positive integer".to_string())
    }
}

#[derive(Deserialize, Debug)]
struct Cursor {
    partition: String,
    offset: String
}

#[derive(Deserialize, Debug)]
struct EventBatch {
    cursor: Cursor,
    events: Option<Vec<Map<String, Value>>>
}
