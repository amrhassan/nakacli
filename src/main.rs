#![feature(conservative_impl_trait)]

extern crate futures;
extern crate tokio_core;
extern crate hyper;
extern crate hyper_tls;
extern crate ansi_term;
extern crate clap;
extern crate serde_json;

mod app;
mod http;
mod fail;
mod server;
mod command_metrics;

use std::process::exit;
use clap::{App, Arg, SubCommand, AppSettings};
use app::Application;
use server::ServerInfo;

fn main() {

    const BEARER_TOKEN_ARG_NAME: &str = "bearer_token";
    const URL_BASE_ARG_NAME: &str = "url_base";
    const METRICS_SUBCOMMAND_NAME: &str = "metrics";
    let matches = App::new("CLI client for Nakadi")
        .setting(AppSettings::SubcommandRequired)
        .arg(Arg::with_name(BEARER_TOKEN_ARG_NAME).long("bearer-token").value_name("TOKEN").help("Bearer token value").takes_value(true).global(true))
        .arg(Arg::with_name(URL_BASE_ARG_NAME).long("url").value_name("NAKADI_URL_BASE").help("scheme://hostname:[port] of the Nakadi server").takes_value(true).global(true))
        .subcommand(SubCommand::with_name(METRICS_SUBCOMMAND_NAME).about("Gets monitoring metrics"))
        .get_matches();

    let server_info = ServerInfo {
        url_base: matches.value_of(URL_BASE_ARG_NAME).unwrap_or("http://localhost"),
        bearer_token: matches.value_of(BEARER_TOKEN_ARG_NAME)
    };

    let mut application = Application::new();

    let action = match matches.subcommand_name() {
        Some(METRICS_SUBCOMMAND_NAME) => command_metrics::metrics(server_info, &mut application),
        _ => panic!("No subcommand is provided, but clap should have handled this already!")
    };

    match application.run(action) {
        Ok(output) => {
            println!("{}", serde_json::to_string_pretty(&output).expect("Failed to pretty-format the JSON response"));
        },
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    }
}
