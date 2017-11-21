use clap::{App, SubCommand, ArgMatches, AppSettings};
use app::Application;
use global::GlobalParams;
use command_eventtype_list;
use command_eventtype_create;
use command_eventtype_delete;

pub const NAME: &str = "event-type";

pub fn sub_command<'a>() -> App<'a, 'a> {
    SubCommand::with_name(NAME).about("Event types")
        .subcommand(command_eventtype_create::sub_command())
        .subcommand(command_eventtype_list::sub_command())
        .subcommand(command_eventtype_delete::sub_command())
        .setting(AppSettings::SubcommandRequired)
}

pub fn run(application: &mut Application, global_params: &GlobalParams, matches: &ArgMatches) {
    if let Some(matches) = matches.subcommand_matches(command_eventtype_create::NAME) {
        command_eventtype_create::run(application, global_params, matches)
    } else if let Some(_) = matches.subcommand_matches(command_eventtype_list::NAME) {
        command_eventtype_list::run(application, global_params)
    } else if let Some(matches) = matches.subcommand_matches(command_eventtype_delete::NAME) {
        command_eventtype_delete::run(application, global_params, matches)
    } else {
        panic!("No command matched! Should have been caught by clap")
    }
}
