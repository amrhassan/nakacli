#![feature(conservative_impl_trait)]

extern crate futures;
extern crate tokio_core;
extern crate hyper;
extern crate hyper_tls;
extern crate ansi_term;
extern crate clap;
extern crate serde_json;

use std::process::exit;
use tokio_core::reactor::Core;
use futures::Future;
use futures::Stream;
use futures::future;
use hyper_tls::HttpsConnector;
use hyper::client::HttpConnector;
use ansi_term::Colour;
use clap::{App, Arg, SubCommand, AppSettings};
use hyper::{Method, Request, Response, Client, Uri};
use hyper::header;
use serde_json::Value;

const DNS_WORKER_THREADS: usize = 4;

fn main() {

    const OAUTH2_TOKEN_ARG_NAME: &str = "oauth2_token";
    const URL_BASE_ARG_NAME: &str = "url_base";
    const METRICS_SUBCOMMAND_NAME: &str = "metrics";
    let matches = App::new("CLI client for Nakadi")
        .setting(AppSettings::SubcommandRequired)
        .arg(Arg::with_name(OAUTH2_TOKEN_ARG_NAME).long("oauth2-token").value_name("TOKEN").help("OAuth2 token value").takes_value(true).global(true))
        .arg(Arg::with_name(URL_BASE_ARG_NAME).long("url").value_name("NAKADI_URL_BASE").help("scheme://hostname:[port] of the Nakadi server").takes_value(true).global(true))
        .subcommand(SubCommand::with_name(METRICS_SUBCOMMAND_NAME).about("Gets monitoring metrics"))
        .get_matches();

    let url_base = matches.value_of(URL_BASE_ARG_NAME).unwrap_or("http://localhost");
    let auth = matches.value_of(OAUTH2_TOKEN_ARG_NAME);

    let mut core = Core::new().expect("Failed to initialize HTTP client event loop");
    let handle = core.handle();
    let client = Client::configure()
        .connector(HttpsConnector::new(DNS_WORKER_THREADS, &handle).expect("Failed to initialize TLS for HTTPS"))
        .build(&handle);

    if let Some(_) = matches.subcommand_matches("metrics") {
        let request = build_request(Method::Get, format!("{}/metrics", url_base).parse().expect("Failed to construct path for HTTP request"), auth, None);
        let response_future = execute_request(&client, request);
        let response_body = response_future.and_then(read_full_resp_body_utf8);

        let action = response_body;

        match core.run(action) {
            Ok(output) => {
                println!("{}", serde_json::to_string_pretty(&output).expect("Failed to pretty-format the JSON response"));
            },
            Err(err) => {
                eprintln!("{}", err);
                exit(1);
            }
        }
    }
}

fn execute_request(http_client: &Client<HttpsConnector<HttpConnector>>, request: Request) -> impl Future<Item=Response, Error=String> {
    http_client
        .request(request)
        .map_err(|err| format!("{}: {}", Colour::Red.paint("Sending HTTP request failed"), err))
}

fn build_request(method: Method, uri: Uri, oauth2_token: Option<&str>, body: Option<&str>) -> Request {
    let mut request = Request::new(method, uri);
    if let Some(token_value) = oauth2_token {
        request.headers_mut().set(header::Authorization("Bearer".to_owned() + token_value))
    }
    if let Some(body_value) = body {
        request.set_body(body_value.to_owned());
    }
    request
}

fn read_full_resp_body_utf8(response: hyper::Response) -> impl Future<Item=Value, Error=String> {
    response
        .body()
        .concat2()
        .map_err(|err| format!("{}: {}", Colour::Red.paint("HTTP error"), err))
        .and_then(|chunk| future::result(String::from_utf8(chunk.into_iter().collect()).map_err(|err| format!("{}", err))))
        .and_then(|text| future::result(serde_json::from_str(&text).map_err(|err| format!("{}", err))))
}
