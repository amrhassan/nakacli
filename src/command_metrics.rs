
use http;
use futures::Future;
use hyper::Method;
use server::ServerInfo;
use serde_json::Value;
use app::Application;
use fail::Failure;

/// Retrieves monitoring metrics information from server
pub fn metrics(
    server_info: ServerInfo,
    application: &mut Application) -> impl Future<Item=Value, Error=Failure> {
    let request = http::build_request(
        Method::Get,
        format!("{}/metrics", server_info.url_base).parse().expect("Failed to construct path for HTTP request"),
        server_info.oauth2_token,
        None);
    let response_future = application.execute_request(request);
    let response_body = response_future.and_then(http::read_full_resp_body_utf8_json);

    response_body
}
