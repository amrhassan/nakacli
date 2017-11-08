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

fn main() {

    let app = App::new("CLI Client for Nakadi")
        .setting(AppSettings::SubcommandRequired)
        .args(global::global_args().as_slice())
        .subcommand(command_metrics::sub_command())
        .subcommand(command_event::sub_command());

    let matches = app.get_matches();

    let global_params = global::extract_global_params(&matches);

    let mut application = Application::new();

    if let Some(matches) = matches.subcommand_matches(command_metrics::NAME) {
        command_metrics::run(&mut application, &global_params, matches)
    } else if let Some(matches) = matches.subcommand_matches(command_event::NAME) {
        command_event::run(&mut application, &global_params, matches)
    } else {
        panic!("No command matched!")
    }
}

