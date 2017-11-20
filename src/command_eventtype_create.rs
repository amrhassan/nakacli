
use clap::{App, ArgMatches, SubCommand, Arg};
use app::Application;
use global::GlobalParams;
use hyper::{Method, StatusCode};
use output;
use server::ServerInfo;
use http;

pub const NAME:                 &'static str = "create";

const ARG_NAME:                 &'static str = "name";
const ARG_OWNING_APPLICATION:   &'static str = "owning-application";
const ARG_CATEGORY:             &'static str = "category";
const ARG_CATEGORY_VALUES       : &'static [&'static str] = &["business", "data", "undefined"];
const ARG_JSON_SCHEMA:          &'static str = "json-schema";


pub fn sub_command<'a>() -> App<'a, 'a> {
    SubCommand::with_name(NAME)
        .about("Creates a new event type")
        .arg(Arg::with_name(ARG_OWNING_APPLICATION).index(1).required(true).help("The owning application ID"))
        .arg(Arg::with_name(ARG_NAME).index(2).required(true).help("The name of the event type"))
        .arg(Arg::with_name(ARG_JSON_SCHEMA).index(3).required(true).help("The JSON Schema of the event"))
        .arg(Arg::with_name(ARG_CATEGORY).long("category").takes_value(true).required(false).possible_values(ARG_CATEGORY_VALUES))
}

struct Params<'a> {
    name: &'a str,
    owning_application: &'a str,
    category: Option<&'a str>,
    json_schema: &'a str,
}

fn extract_params<'a>(matches: &'a ArgMatches) -> Params<'a> {
    Params {
        name: matches.value_of(ARG_NAME).expect("Non-optional argument should have been caught by clap if missing"),
        owning_application: matches.value_of(ARG_OWNING_APPLICATION).expect("Non-optional argument should have been caught by clap if missing"),
        category: matches.value_of(ARG_CATEGORY),
        json_schema: matches.value_of(ARG_JSON_SCHEMA).expect("Non-optional argument should have been caught by clap if missing"),
    }
}

pub fn run(application: &mut Application, global_params: &GlobalParams, matches: &ArgMatches) {

    let server_info = ServerInfo::from_params(global_params);

    let params = extract_params(matches);

    let request_body = json!({
        "name": params.name.to_string(),
        "owning_application": params.owning_application.to_string(),
        "category": params.category.unwrap_or("undefined").to_string(),
        "schema": {
            "type": "json_schema",
            "schema": params.json_schema.to_string()
        }
    });

    let action = http::execute_and_read_full_resp_body_utf8(
        &application.http_client,
        Method::Post,
        "/event-types",
        &server_info,
        Some(&request_body)
    );

    let result = application.core.run(action);
    output::final_result(result, StatusCode::Created, global_params.pretty)
}
