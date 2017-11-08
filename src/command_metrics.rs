
use http;
use hyper::{Method, StatusCode};
use server::ServerInfo;
use app::Application;
use output;
use global::*;
use clap::*;

pub const NAME: &'static str = "metrics";

pub fn sub_command() -> App<'static, 'static> {
    SubCommand::with_name(NAME).about("Gets monitoring metrics")
}

struct Params;

fn extract_params(_matches: &ArgMatches) -> Params {
    Params {}
}

pub fn run(application: &mut Application, global_params: &GlobalParams, _matches: &ArgMatches) {
    let _params = extract_params(_matches);
    let server_info = ServerInfo::from_params(global_params);
    let action = http::execute_and_read_full_resp_body_utf8(
        &application.http_client,
        Method::Get,
        "/metrics",
        &server_info,
        None
    );
    let result = application.core.run(action);
    output::final_result(result, StatusCode::Ok, global_params.pretty)
}
