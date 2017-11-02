
use ansi_term::Colour;
use std::fmt::{Display, Formatter, Result};
use std::process::exit;

pub struct Failure { show: String }

impl Display for Failure {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.show)
    }
}

/// Canonical representation of error message
pub fn failure<A: Display>(header: &str, detailed: A) -> Failure {
    Failure { show: format!("{}, {}", Colour::Red.paint(header), detailed) }
}

pub fn die(exit_code: i32, failure: Failure) -> ! {
    eprintln!("{}", failure);
    exit(exit_code)
}
