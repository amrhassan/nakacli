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
mod command_event;
mod command_event_publish;
mod command_event_stream;
mod auth;
mod output;
mod global;

use clap::{App, AppSettings};
use app::Application;
use server::ServerInfo;

fn main() {

    let app = App::new("CLI Client for Nakadi")
        .setting(AppSettings::SubcommandRequired)
        .args(global::global_args().as_slice())
        .subcommand(command_metrics::sub_command())
        .subcommand(command_event::sub_command());

    let matches = app.get_matches();

    let global_params = global::extract_global_params(&matches);

    let authorization =
        if global_params.zign {
            server::Authorization::Zign
        } else if let Some(bearer_token) = global_params.bearer_token {
            server::Authorization::BearerToken(bearer_token)
        } else {
            server::Authorization::None
        };

    let server_info = ServerInfo {
        url_base: global_params.nakadi_url.unwrap_or("http://localhost"),
        authorization,
    };

    let mut application = Application::new();

    if let Some(matches) = matches.subcommand_matches(command_metrics::NAME) {
        let params = command_metrics::extract_params(matches);
        command_metrics::run(&server_info, &mut application, &global_params, params)
    } else if let Some(matches) = matches.subcommand_matches(command_event::NAME) {
        let params = command_event::extract_params(matches);
        command_event::run(&server_info, &mut application, &global_params, params)
    } else {
        panic!("No command matched!")
    }
}

