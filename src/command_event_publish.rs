
use server::ServerInfo;
use app::Application;
use serde_json;
use http;
use hyper::{Method, StatusCode};
use output;

pub fn run(server_info: &ServerInfo, application: &mut Application, event_type: &str, json_body: &str, pretty: bool) {
    let path = format!("/event-types/{}/events", event_type);
    let body = {
        let decoded = serde_json::from_str::<serde_json::Value>(json_body).expect("Failed to JSON-decode text that was validated to be JSON by clap");
        let json_array = if decoded.is_object() {
            serde_json::Value::Array(vec![decoded])
        } else if decoded.is_array() {
            decoded
        } else {
            panic!("Input json_body should have been validated to be JSON object or array by clap")
        };
        serde_json::to_string(&json_array).expect("Failed to encode into JSON for some reason!")
    };

    let action = http::execute_and_read_full_resp_body_utf8(
        &application.http_client,
        Method::Post,
        &path,
        server_info,
        Some(&body)
    );
    let result = application.core.run(action);
    output::final_result(result, StatusCode::Ok, pretty)
}

pub fn validate_json_body(value: String) -> Result<(), String> {
    match serde_json::from_str::<serde_json::Value>(&value) {
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

