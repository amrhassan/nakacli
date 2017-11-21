
use clap::{App, ArgMatches, SubCommand, Arg};
use app::Application;
use global::GlobalParams;
use hyper::{Method, StatusCode};
use output;
use server::ServerInfo;
use http;

pub const NAME:                         &'static str = "delete";
const ARG_NAME:                         &'static str = "name";

pub fn sub_command<'a>() -> App<'a, 'a> {
    SubCommand::with_name(NAME)
        .about("Deletes an event type")
        .arg(Arg::with_name(ARG_NAME).required(true).help("The name of the event type"))
}

struct Params<'a> {
    name: &'a str,
}

fn extract_params<'a>(matches: &'a ArgMatches) -> Params<'a> {
    Params {
        name: matches.value_of(ARG_NAME).expect("Non-optional argument should have been caught by clap if missing"),
    }
}

pub fn run(application: &mut Application, global_params: &GlobalParams, matches: &ArgMatches) {

    let server_info = ServerInfo::from_params(global_params);

    let params = extract_params(matches);

    let path = format!("/event-types/{}", params.name);

    let action = http::execute_and_read_full_resp_body_utf8(
        &application.http_client,
        Method::Delete,
        &path,
        &server_info,
        None
    );

    let result = application.core.run(action);
    output::final_result(result, StatusCode::Ok, global_params.pretty)
}
