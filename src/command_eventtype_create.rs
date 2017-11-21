
use clap::{App, ArgMatches, SubCommand, Arg};
use app::Application;
use global::GlobalParams;
use hyper::{Method, StatusCode};
use output;
use server::ServerInfo;
use http;
use input::long_argument;

pub const NAME:                         &str = "create";

const ARG_NAME:                         &str = "name";
const ARG_OWNING_APPLICATION:           &str = "owning-application";
const ARG_CATEGORY:                     &str = "category";
const ARG_CATEGORY_VALUES:              &'static [&str] = &["undefined", "business", "data"];
const ARG_JSON_SCHEMA:                  &str = "json-schema";
const ARG_PARTITION_STRATEGY:           &str = "partition-strategy";
const ARG_PARTITION_STRATEGY_VALUES:    &'static [&str] = &["random", "hash"];
const ARG_COMPATIBILITY_MODE:           &str = "compatibility-mode";
const ARG_COMPATIBILITY_MODE_VALUES:    &[&str] = &["forward", "compatible", "none"];
const ARG_PARTITION_KEY_FIELDS:         &str = "partition-key-fields";


pub fn sub_command<'a>() -> App<'a, 'a> {
    SubCommand::with_name(NAME)
        .about("Creates a new event type")
        .arg(Arg::with_name(ARG_OWNING_APPLICATION).index(1).required(true).help("The owning application ID"))
        .arg(Arg::with_name(ARG_NAME).index(2).required(true).help("The name of the event type"))
        .arg(Arg::with_name(ARG_JSON_SCHEMA)
            .index(3)
            .required(true)
            .validator(validate_json_schema)
            .help("The JSON Schema of the event type (Use '@' prefix to specify a filepath. e.g. '@schema.json')"))
        .arg(Arg::with_name(ARG_CATEGORY)
            .long("category")
            .takes_value(true)
            .required(false)
            .possible_values(ARG_CATEGORY_VALUES)
            .default_value(ARG_CATEGORY_VALUES[0])
        )
        .arg(Arg::with_name(ARG_PARTITION_STRATEGY)
            .long("partition-strategy")
            .takes_value(true)
            .required(false)
            .possible_values(ARG_PARTITION_STRATEGY_VALUES)
            .default_value(ARG_PARTITION_STRATEGY_VALUES[0])
        )
        .arg(Arg::with_name(ARG_COMPATIBILITY_MODE)
            .long("compatibility-mode")
            .takes_value(true)
            .required(false)
            .possible_values(ARG_COMPATIBILITY_MODE_VALUES)
            .default_value(ARG_COMPATIBILITY_MODE_VALUES[0])
        )
        .arg(Arg::with_name(ARG_PARTITION_KEY_FIELDS)
            .multiple(true)
            .long("partition-key-field")
            .takes_value(true)
            .required_if(ARG_PARTITION_STRATEGY, "hash")
        )
}

struct Params<'a> {
    name: &'a str,
    owning_application: &'a str,
    category: &'a str,
    json_schema: String,
    compatibility_mode: &'a str,
    partition_strategy: &'a str,
    partition_key_fields: Option<Vec<&'a str>>,
}

fn extract_params<'a>(matches: &'a ArgMatches) -> Params<'a> {
    Params {
        name: matches.value_of(ARG_NAME).expect("Non-optional argument should have been caught by clap if missing"),
        owning_application: matches.value_of(ARG_OWNING_APPLICATION).expect("Non-optional argument should have been caught by clap if missing"),
        category: matches.value_of(ARG_CATEGORY).expect("Non-optional argument should have been caught by clap if missing"),
        json_schema: matches.value_of(ARG_JSON_SCHEMA).and_then(|v| long_argument(v).ok()).expect("Non-optional argument should have been caught by clap if missing"),
        compatibility_mode: matches.value_of(ARG_COMPATIBILITY_MODE).expect("Non-optional argument should have been caught by clap if missing"),
        partition_strategy: matches.value_of(ARG_PARTITION_STRATEGY).expect("Non-optional argument should have been caught by clap if missing"),
        partition_key_fields: matches.values_of(ARG_PARTITION_KEY_FIELDS).map(|values| values.collect()),
    }
}

pub fn run(application: &mut Application, global_params: &GlobalParams, matches: &ArgMatches) {

    let server_info = ServerInfo::from_params(global_params);

    let params = extract_params(matches);

    let request_body = json!({
        "name": params.name.to_string(),
        "owning_application": params.owning_application.to_string(),
        "category": params.category.to_string(),
        "partition_strategy": params.partition_strategy,
        "compatibility_mode": params.compatibility_mode,
        "partition_key_fields": params.partition_key_fields,
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

pub fn validate_json_schema(value: String) -> Result<(), String> {
    long_argument(&value).map(|_| ())
}
