
use serde_json;
use serde_json::{Value, to_string_pretty, from_str, to_string};
use hyper::StatusCode;
use ansi_term::Colour;
use std::fmt::{Display, Formatter};
use std::process::exit;
use std::fmt;

/// Exits the application with failure
pub fn die_failure(failure: Failure) -> ! {
    eprintln!("{}", failure);
    exit(1)
}

/// Exits the application with success
pub fn die_success() -> ! {
    exit(0)
}

/// Prints operation result from the Nakadi server, then exits either with success or failure based on the `expected_status_code`.
pub fn final_result(result: Result<(StatusCode, String), Failure>, expected_status_code: StatusCode, pretty: bool) {
    match result {
        Ok((status_code, ref output)) if status_code == expected_status_code => {
            if !output.is_empty() {
                print_json(&output, pretty);
            }
            die_success()
        },
        Ok((_, output)) => {
            if pretty {
                die_failure(failureln("Unexpected response:", pretty_json(&output)));
            } else {
                die_failure(failure("Unexpected response:", &output));
            }
        }
        Err(err) => {
            die_failure(err);
        }
    }
}

/// Prints a JSON value encoded as a String
pub fn print_json(result: &str, pretty: bool) {
    if pretty {
        println!("{}", pretty_json(result))
    } else {
        println!("{}", result)
    }
}

/// Prints a JSON value
pub fn print_json_value(value: &Value, pretty: bool) {
    if pretty {
        println!("{}", to_string_pretty(value).expect("Failed to serialize a JSON value"))
    } else {
        println!("{}", to_string(value).expect("Failed to serialize a JSON value"))
    }
}

/// Canonical representation of error message
pub fn failure<A: Display>(header: &str, detailed: A) -> Failure {
    Failure { show: format!("{}, {}", Colour::Red.paint(header), detailed) }
}

pub struct Failure { show: String }

impl Display for Failure {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.show)
    }
}

/// Constructs a `Failure` that spans two lines
fn failureln<A: Display>(header: &str, detailed: A) -> Failure {
    Failure { show: format!("{}\n{}", Colour::Red.paint(header), detailed) }
}

fn pretty_json(json: &str) -> String {
    let value: Result<Value, serde_json::Error> = from_str(json);
    value.and_then(|ref v| to_string_pretty(v)).unwrap_or(json.to_owned())
}
