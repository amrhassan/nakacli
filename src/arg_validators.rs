
use serde_json;
use input::long_argument;

pub fn unsigned_int(v: String) -> Result<(), String> {
    match v.parse::<u64>() {
        Ok(n) if n > 0 => Ok(()),
        _ => Err("Value should be a positive integer".to_string())
    }
}

pub fn json(value: String) -> Result<(), String> {
    match serde_json::from_str::<serde_json::Value>(&long_argument(&value)?) {
        Err(err) => Err(format!("JSON body of event is malformed: {}", err)),
        Ok(json_value) =>
            if let Some(array) = json_value.as_array() {
                if array.iter().all(|obj| obj.is_object()) {
                    Ok(())
                } else {
                    Err(format!("JSON body should be an array of objects if you're trying to publish multiple events"))
                }
            } else if !json_value.is_object() {
                Err("JSON body of event needs to be an object".to_owned())
            } else {
                Ok(())
            }
    }
}
