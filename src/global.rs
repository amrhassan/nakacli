
use clap::{Arg, ArgMatches};

const ARG_PRETTY: &str = "pretty";
const ARG_ZIGN: &str = "zign";
const ARG_BEARER_TOKEN: &str = "bearer_token";
const ARG_NAKADI_URL: &str = "nakadi_url";

pub struct GlobalParams<'a> {
    pub pretty: bool,
    pub zign: bool,
    pub bearer_token: Option<&'a str>,
    pub nakadi_url: Option<&'a str>
}

pub fn extract_global_params<'a>(matches: &'a ArgMatches) -> GlobalParams<'a> {
    GlobalParams {
        pretty: matches.occurrences_of(ARG_PRETTY) > 0,
        zign: matches.occurrences_of(ARG_ZIGN) > 0,
        bearer_token: matches.value_of(ARG_BEARER_TOKEN),
        nakadi_url: matches.value_of(ARG_NAKADI_URL)
    }
}

pub fn global_args() -> Vec<Arg<'static, 'static>> {

    let bearer_token = Arg::with_name(ARG_BEARER_TOKEN)
        .long("bearer-token")
        .value_name("TOKEN")
        .help("Bearer token value")
        .env("BEARER_TOKEN")
        .global(true)
        .conflicts_with(ARG_ZIGN);

    let nakadi_url = Arg::with_name(ARG_NAKADI_URL)
        .long("url")
        .value_name("NAKADI_URL")
        .help("scheme://hostname:[port] of the Nakadi server")
        .env("NAKADI_URL").global(true);

    let zign = Arg::with_name(ARG_ZIGN)
        .long("zign")
        .help("Use zign to acquire a Bearer token")
        .takes_value(false)
        .global(true)
        .conflicts_with(ARG_BEARER_TOKEN);

    let pretty = Arg::with_name(ARG_PRETTY)
        .long("pretty")
        .help("Prints pretty JSON output")
        .global(true)
        .takes_value(false);

    vec![
        bearer_token,
        nakadi_url,
        zign,
        pretty
    ]
}
