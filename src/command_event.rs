
use clap::{App, SubCommand, ArgMatches};
use command_event_publish;
use command_event_stream;
use app::Application;
use global::GlobalParams;

pub const NAME: &'static str = "event";

pub fn sub_command() -> App<'static, 'static> {
    SubCommand::with_name(NAME).about("Events of a certain type")
        .subcommand(command_event_publish::sub_command())
        .subcommand(command_event_stream::sub_command())
}

pub fn run(application: &mut Application, global_params: &GlobalParams, matches: &ArgMatches) {
    if let Some(matches) = matches.subcommand_matches(command_event_publish::NAME) {
        command_event_publish::run(application, global_params, matches)
    } else if let Some(matches) = matches.subcommand_matches(command_event_stream::NAME) {
        command_event_stream::run(application, global_params, matches)
    } else {
        panic!("A subcommand was not provided!")
    }
}
