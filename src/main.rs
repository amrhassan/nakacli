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
mod auth;

use clap::{App, Arg, SubCommand, AppSettings};
use app::Application;
use server::ServerInfo;
use fail::die;

fn main() {

    const BEARER_TOKEN_ARG_NAME: &str = "bearer_token";
    const NAKADI_URL_ARG_NAME: &str = "nakadi_url";
    const METRICS_SUBCOMMAND_NAME: &str = "metrics";
    const ZIGN_ARG_NAME: &str = "zign";
    let matches = App::new("CLI Client for Nakadi")
        .setting(AppSettings::SubcommandRequired)
        .arg(Arg::with_name(BEARER_TOKEN_ARG_NAME).long("bearer-token").value_name("TOKEN").help("Bearer token value").env("BEARER_TOKEN").global(true).conflicts_with(ZIGN_ARG_NAME))
        .arg(Arg::with_name(NAKADI_URL_ARG_NAME).long("url").value_name("NAKADI_URL_BASE").help("scheme://hostname:[port] of the Nakadi server").env("NAKADI_URL").global(true))
        .arg(Arg::with_name(ZIGN_ARG_NAME).long("zign").help("Use zign to acquire a Bearer token").takes_value(false).global(true).conflicts_with(BEARER_TOKEN_ARG_NAME))
        .subcommand(SubCommand::with_name(METRICS_SUBCOMMAND_NAME).about("Gets monitoring metrics"))
        .get_matches();

    let authorization =
        if matches.occurrences_of(ZIGN_ARG_NAME) > 0 {
            server::Authorization::Zign
        } else if let Some(bearer_token) = matches.value_of(BEARER_TOKEN_ARG_NAME) {
            server::Authorization::BearerToken(bearer_token)
        } else {
            server::Authorization::None
        };

    let server_info = ServerInfo {
        url_base: matches.value_of(NAKADI_URL_ARG_NAME).unwrap_or("http://localhost"),
        authorization,
    };

    let mut application = Application::new();

    let action = match matches.subcommand_name() {
        Some(METRICS_SUBCOMMAND_NAME) => command_metrics::metrics(server_info, &mut application),
        _ => panic!("No subcommand is provided, but clap should have handled this already!")
    };

    match action {
        Ok(action_future) => match application.run(action_future) {
            Ok(output) => {
                println!("{}", serde_json::to_string_pretty(&output).expect("Failed to pretty-format the JSON response"));
            },
            Err(err) => {
                die(1, err);
            }
        },
        Err(err) => die(1, err)
    }
}
