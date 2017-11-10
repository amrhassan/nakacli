use http::*;
use futures::future;
use server::ServerInfo;
use futures::Future;
use app::Application;
use hyper::{Method, Response, StatusCode};
use output::{die, failure, print_json_value, Failure};
use futures::Stream;
use serde_json::{Value, Map, from_str};
use global::*;
use clap::{ArgMatches, App, SubCommand, Arg};
use futures::stream;

pub const NAME: &'static str = "stream";
const ARG_EVENT_TYPE: &'static str = "event-type";

struct Params<'a> {
    event_type: &'a str
}

fn extract_params<'a>(matches: &'a ArgMatches) -> Params<'a> {
    Params {
        event_type: matches.value_of(ARG_EVENT_TYPE).expect("Non-optional argument should have been caught by clap if missing")
    }
}

pub fn sub_command<'a>() -> App<'a, 'a> {
    SubCommand::with_name(NAME)
        .about("Stream-listen on published events")
        .arg(Arg::with_name(ARG_EVENT_TYPE).required(true).index(1).help("Name of the Event Type"))
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
        .and_then(|resp| process_response(resp, global_params));
    match application.core.run(action) {
        Err(err) => die(1, err),
        Ok(_) => ()
    }
}

fn process_response<'a>(resp: Response, global_params: &'a GlobalParams<'a>) -> impl Future<Item=(), Error=Failure> + 'a {
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
            .fold(Vec::new(), move |acc, byte| {
                if byte == b'\n' {
                    let line = String::from_utf8(acc).expect("Failed to UTF-8 decode the response");

                    let batch: EventBatch = {
                        match from_str(&line) {
                            Err(err) => die(1, failure("Failed to decode an event stream batch", err)),
                            Ok(batch) => batch
                        }
                    };

                    if let Some(events) = batch.events {
                        for event in events {
                            print_json_value(&Value::Object(event), global_params.clone().pretty)
                        }
                    }

                    future::ok(Vec::new())
                } else {
                    let mut mut_acc = acc;
                    mut_acc.push(byte);
                    future::ok(mut_acc)
                }
            })
            .map(|leftover_bytes| if leftover_bytes.is_empty() { panic!("Leftover bytes") })
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
