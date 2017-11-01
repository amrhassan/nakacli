
use ansi_term::Colour;
use std::fmt::Display;

/// Canonical representation of error message
pub fn failure<A: Display>(header: &str, detailed: A) -> String {
    format!("{}, {}", Colour::Red.paint(header), detailed)
}