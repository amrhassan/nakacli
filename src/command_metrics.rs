
use http;
use hyper::{Method, StatusCode};
use server::ServerInfo;
use app::Application;
use output;
use global::*;
use clap::*;

pub const NAME: &'static str = "metrics";

pub  fn sub_command<'a>() -> App<'a, 'a> {
    SubCommand::with_name(NAME).about("Gets monitoring metrics")
}

pub fn run(application: &mut Application, global_params: &GlobalParams) {
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
