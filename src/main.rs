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
mod output;

use clap::{App, Arg, SubCommand, AppSettings};
use app::Application;
use server::ServerInfo;

fn main() {

    struct FlagName;
    impl FlagName {
        const ZIGN: &'static str = "zign";
    }

    struct OptionName;
    impl OptionName {
        const BEARER_TOKEN: &'static str = "bearer_token";
        const NAKADI_URL: &'static str = "nakadi_url";
    }

    struct SubCommandName;
    impl SubCommandName {
        const METRICS: &'static str = "metrics";
        const EVENT: &'static str = "event";
        const EVENT_PUBLISH: &'static str = "publish";
    }

    let option_bearer_token = Arg::with_name(OptionName::BEARER_TOKEN)
        .long("bearer-token")
        .value_name("TOKEN")
        .help("Bearer token value")
        .env("BEARER_TOKEN")
        .global(true)
        .conflicts_with(FlagName::ZIGN);

    let option_nakadi_url = Arg::with_name(OptionName::NAKADI_URL)
        .long("url")
        .value_name("NAKADI_URL_BASE")
        .help("scheme://hostname:[port] of the Nakadi server")
        .env("NAKADI_URL").global(true);

    let flag_zign = Arg::with_name(FlagName::ZIGN)
        .long("zign")
        .help("Use zign to acquire a Bearer token")
        .takes_value(false)
        .global(true)
        .conflicts_with(OptionName::BEARER_TOKEN);

    let subcommand_metrics = SubCommand::with_name(SubCommandName::METRICS)
        .about("Gets monitoring metrics");

    let subcommand_events_publish = SubCommand::with_name(SubCommandName::EVENT_PUBLISH).about("Publish events");

    let subcommand_events = SubCommand::with_name(SubCommandName::EVENT).about("Events of a certain type").subcommand(subcommand_events_publish);

    let app = App::new("CLI Client for Nakadi")
        .setting(AppSettings::SubcommandRequired)
        .arg(option_bearer_token)
        .arg(option_nakadi_url)
        .arg(flag_zign)
        .subcommand(subcommand_metrics)
        .subcommand(subcommand_events);

    let matches = app.get_matches();

    let authorization =
        if matches.occurrences_of(FlagName::ZIGN) > 0 {
            server::Authorization::Zign
        } else if let Some(bearer_token) = matches.value_of(OptionName::BEARER_TOKEN) {
            server::Authorization::BearerToken(bearer_token)
        } else {
            server::Authorization::None
        };

    let server_info = ServerInfo {
        url_base: matches.value_of(OptionName::NAKADI_URL).unwrap_or("http://localhost"),
        authorization,
    };

    let mut application = Application::new();

    match matches.subcommand_name() {
        Some(SubCommandName::METRICS) => command_metrics::run(server_info, &mut application),
        _ => panic!("No subcommand is provided, but clap should have handled this already!")
    }
}
