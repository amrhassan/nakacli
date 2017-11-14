
use clap::{App, ArgMatches};
use app::Application;
use global::GlobalParams;

pub const NAME: &'static str = "list";

pub fn sub_command<'a>() -> App<'a, 'a> {
    unimplemented!()
}

pub fn run(_application: &mut Application, _global_params: &GlobalParams, _matches: &ArgMatches) {
    unimplemented!()
}
