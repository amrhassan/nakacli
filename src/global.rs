
use clap::{Arg, ArgMatches};
use std::time::Duration;
use arg_validators;

const ARG_PRETTY: &str = "pretty";
const ARG_ZIGN: &str = "zign";
const ARG_BEARER_TOKEN: &str = "bearer_token";
const ARG_NAKADI_URL: &str = "nakadi_url";
const ARG_NETWORK_TIMEOUT: &str = "network-timeout";

pub struct GlobalParams<'a> {
    pub pretty: bool,
    pub zign: bool,
    pub bearer_token: Option<&'a str>,
    pub nakadi_url: Option<&'a str>,
    pub network_timeout: Option<Duration>,
}

pub fn extract_global_params<'a>(matches: &'a ArgMatches) -> GlobalParams<'a> {
    GlobalParams {
        pretty: matches.occurrences_of(ARG_PRETTY) > 0,
        zign: matches.occurrences_of(ARG_ZIGN) > 0,
        bearer_token: matches.value_of(ARG_BEARER_TOKEN),
        nakadi_url: matches.value_of(ARG_NAKADI_URL),
        network_timeout: matches.value_of(ARG_NETWORK_TIMEOUT).map(|v| Duration::from_secs(v.parse::<u64>().expect("Invalid u64 that should have been caught by clap"))),
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

    let network_timeout = Arg::with_name(ARG_NETWORK_TIMEOUT)
        .long("network-timeout")
        .help("Network timeout for non-streaming operations (in seconds)")
        .global(true)
        .takes_value(true)
        .default_value("1")
        .validator(arg_validators::unsigned_int);

    vec![
        bearer_token,
        nakadi_url,
        zign,
        pretty,
        network_timeout,
    ]
}
