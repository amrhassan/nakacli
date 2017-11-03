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

use clap::{App, Arg, SubCommand, AppSettings};
use app::Application;
use server::ServerInfo;
use std::process::Command;
use fail::{Failure, failure, die};

fn main() {

    const BEARER_TOKEN_ARG_NAME: &str = "bearer_token";
    const URL_BASE_ARG_NAME: &str = "url_base";
    const METRICS_SUBCOMMAND_NAME: &str = "metrics";
    const ZIGN_ARG_NAME: &str = "zign";
    let matches = App::new("CLI Client for Nakadi")
        .setting(AppSettings::SubcommandRequired)
        .arg(Arg::with_name(BEARER_TOKEN_ARG_NAME).long("bearer-token").value_name("TOKEN").help("Bearer token value").env("BEARER_TOKEN").global(true).conflicts_with(ZIGN_ARG_NAME))
        .arg(Arg::with_name(URL_BASE_ARG_NAME).long("url").value_name("NAKADI_URL_BASE").help("scheme://hostname:[port] of the Nakadi server").env("NAKADI_URL").global(true))
        .arg(Arg::with_name(ZIGN_ARG_NAME).long("zign").help("Use zign to acquire a Bearer token").takes_value(false).global(true).conflicts_with(BEARER_TOKEN_ARG_NAME))
        .subcommand(SubCommand::with_name(METRICS_SUBCOMMAND_NAME).about("Gets monitoring metrics"))
        .get_matches();

    let bearer_token = match matches.value_of(BEARER_TOKEN_ARG_NAME) {
        Some(v) => Some(v.to_owned()),
        None => if matches.occurrences_of(ZIGN_ARG_NAME) > 0 {
            match zign() {
                Err(err) => die(2, err),
                Ok(v) => Some(v)
            }
        } else {
            None
        }
    };

    let server_info = ServerInfo {
        url_base: matches.value_of(URL_BASE_ARG_NAME).unwrap_or("http://localhost"),
        bearer_token: bearer_token.as_ref().map(|s| s.as_ref())
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
            die(1, failure("Unexpected error", err));
        }
    }
}

/// Executes the zign command
fn zign() -> Result<String, Failure> {
    let output = Command::new("zign").arg("token").output().map_err(|err| failure("Failed to run zign command", err))?;
    if !output.status.success() {
        let exit_code = output.status.code().ok_or(failure("zign command was interrupted", ""))?;
        let stderr = String::from_utf8(output.stderr).map_err(|err| failure("Failed to decode zign stderr output as UTF-8", err))?;
        return Err(failure(&format!("zign command failed with exit code {}", exit_code), stderr))
    }
    String::from_utf8(output.stdout).map_err(|err| failure("Failed to decode zign stdout output as UTF-8", err))
}

