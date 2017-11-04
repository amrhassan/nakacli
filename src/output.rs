
use serde_json::*;

pub fn pretty_json(json: &str) -> String {
    let value: Result<Value> = from_str(json);
    value.and_then(|ref v| to_string_pretty(v)).unwrap_or(json.to_owned())
}
