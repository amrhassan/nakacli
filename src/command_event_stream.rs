use http::*;
use futures::future;
use server::ServerInfo;
use futures::Future;
use app::Application;
use hyper::Method;
use output::{die, failure, print_json_value};
use futures::Stream;
use serde_json::{Value, Map, from_str};

pub fn run(server_info: &ServerInfo, application: &mut Application, event_type: &str, pretty: bool) {
    let path = format!("/event-types/{}/events", event_type);
    let method = Method::Get;
    let body = None;
    let request = build_request(method, &path, server_info, body);
    let http_client = &application.http_client;

    let mut buffer = String::new();

    let action = future::result(request)
        .and_then(move |r| execute_request(http_client, r))
        .and_then(|resp| {
            let stream = resp.body();
            stream.for_each(|chunk| {

                // Add new chunk to buffer
                buffer.extend(String::from_utf8(chunk.into_iter().collect()));

                // Split off first line from rest of buffer
                let line_ends_at = {
                    match buffer.find('\n') {
                        None => return Ok(()),
                        Some(n) => n
                    }
                };

                let line = buffer.split_at(line_ends_at).0.to_owned();

                let batch: EventBatch = {
                    match from_str(&line) {
                        Err(err) => die(1, failure("Failed to decode an event stream batch (Malformed/incomplete JSON value in a single HTTP chunk?)", err)),
                        Ok(batch) => batch
                    }
                };

                if let Some(events) = batch.events {
                    for event in events {
                        print_json_value(&Value::Object(event), pretty)
                    }
                }

                // Success, now drops the line from the buffer
                let _ = buffer.drain(..line_ends_at+1).collect::<Vec<char>>();
                Ok(())
            }).map_err(|err| failure("Failed to stream chunks from HTTP server", err))
        });
    match application.core.run(action) {
        Err(err) => die(1, err),
        Ok(_) => ()
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