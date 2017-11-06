
use http;
use hyper::{Method, StatusCode};
use server::ServerInfo;
use app::Application;
use output;

pub fn run(server_info: &ServerInfo, application: &mut Application, pretty: bool) {
    let action = http::execute_and_read_full_resp_body_utf8(
        &application.http_client,
        Method::Get,
        "/metrics",
        server_info,
        None
    );
    let result = application.core.run(action);
    output::final_result(result, StatusCode::Ok, pretty)
}
