#![feature(conservative_impl_trait)]

extern crate futures;
extern crate tokio_core;
extern crate hyper;
extern crate hyper_tls;
extern crate ansi_term;
extern crate clap;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

mod app;
mod http;
mod server;
mod command_metrics;
mod command_event_publish;
mod command_event_stream;
mod auth;
mod output;

use clap::{App, Arg, SubCommand, AppSettings};
use app::Application;
use server::ServerInfo;

fn main() {

    struct FlagName;
    impl FlagName {
        const ZIGN:         &'static str = "zign";
        const PRETTY:       &'static str = "pretty";
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
        const EVENT_STREAM: &'static str = "stream";
    }

    struct ArgName;
    impl ArgName {
        const EVENT_PUBLISH_TYPE: &'static str = "event-type";
        const EVENT_PUBLISH_JSON_BODY: &'static str = "json-body";
        const EVENT_STREAM_TYPE: &'static str = "event-type";
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
        .value_name("NAKADI_URL")
        .help("scheme://hostname:[port] of the Nakadi server")
        .env("NAKADI_URL").global(true);

    let flag_zign = Arg::with_name(FlagName::ZIGN)
        .long("zign")
        .help("Use zign to acquire a Bearer token")
        .takes_value(false)
        .global(true)
        .conflicts_with(OptionName::BEARER_TOKEN);

    let flag_pretty = Arg::with_name(FlagName::PRETTY)
        .long("pretty")
        .help("Prints pretty JSON output")
        .global(true)
        .takes_value(false);

    let subcommand_metrics = SubCommand::with_name(SubCommandName::METRICS)
        .about("Gets monitoring metrics");

    let subcommand_events_publish = SubCommand::with_name(SubCommandName::EVENT_PUBLISH)
        .about("Publish one or more events")
        .arg(Arg::with_name(ArgName::EVENT_PUBLISH_TYPE).required(true).index(1).help("Name of the Event Type"))
        .arg(Arg::with_name(ArgName::EVENT_PUBLISH_JSON_BODY)
            .required(true)
            .index(2)
            .help("Body of one or more events in JSON format")
            .validator(command_event_publish::validate_json_body)
        );

    let subcommand_events_stream = SubCommand::with_name(SubCommandName::EVENT_STREAM)
        .about("Stream-listen on published events")
        .arg(Arg::with_name(ArgName::EVENT_STREAM_TYPE).required(true).index(1).help("Name of the Event Type"));

    let subcommand_events = SubCommand::with_name(SubCommandName::EVENT).about("Events of a certain type")
        .subcommand(subcommand_events_publish)
        .subcommand(subcommand_events_stream);

    let app = App::new("CLI Client for Nakadi")
        .setting(AppSettings::SubcommandRequired)
        .arg(option_bearer_token)
        .arg(option_nakadi_url)
        .arg(flag_zign)
        .arg(flag_pretty)
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

    let pretty = matches.occurrences_of(FlagName::PRETTY) > 0;

    let server_info = ServerInfo {
        url_base: matches.value_of(OptionName::NAKADI_URL).unwrap_or("http://localhost"),
        authorization,
    };

    let mut application = Application::new();

    if let Some(_) = matches.subcommand_matches(SubCommandName::METRICS) {
        command_metrics::run(&server_info, &mut application, pretty)
    } else if let Some(matches) = matches.subcommand_matches(SubCommandName::EVENT) {
        if let Some(matches) = matches.subcommand_matches(SubCommandName::EVENT_PUBLISH) {
            command_event_publish::run(
                &server_info,
                &mut application,
                matches.value_of(ArgName::EVENT_PUBLISH_TYPE).expect("Non-optional argument should have been caught by clap if missing"),
                matches.value_of(ArgName::EVENT_PUBLISH_JSON_BODY).expect("Non-optional argument should have been caught by clap if missing"),
                pretty
            )
        } else if let Some(matches) = matches.subcommand_matches(SubCommandName::EVENT_STREAM) {
            command_event_stream::run(
                &server_info,
                &mut application,
                matches.value_of(ArgName::EVENT_STREAM_TYPE).expect("Non-optional argument should have been caught by clap if missing"),
                pretty
            )
        } else {
            panic!("No command match!")
        }
    } else {
        panic!("No command matched!")
    }
}
