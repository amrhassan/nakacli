
use clap::{App, SubCommand, ArgMatches};
use command_event_publish;
use command_event_stream;
use server::ServerInfo;
use app::Application;
use global::GlobalParams;

pub const NAME: &'static str = "event";

pub fn sub_command() -> App<'static, 'static> {
    SubCommand::with_name(NAME).about("Events of a certain type")
        .subcommand(command_event_publish::sub_command())
        .subcommand(command_event_stream::sub_command())
}

pub struct Params<'a>{
    matches: &'a ArgMatches<'a>
}

pub fn extract_params<'a>(matches: &'a ArgMatches) -> Params<'a> {
    Params {
        matches
    }
}

pub fn run(server_info: &ServerInfo, application: &mut Application, global_params: &GlobalParams, params: Params) {
    if let Some(matches) = params.matches.subcommand_matches(command_event_publish::NAME) {
        let params = command_event_publish::extract_params(matches);
        command_event_publish::run(server_info, application, &params, global_params)
    } else if let Some(matches) = params.matches.subcommand_matches(command_event_stream::NAME) {
        let params = command_event_stream::extract_params(matches);
        command_event_stream::run(server_info, application, &params, global_params)
    } else {
        panic!("A subcommand was not provided!")
    }
}
