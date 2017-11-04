
use http::*;
use futures::Future;
use hyper::Method;
use server::ServerInfo;
use serde_json::Value;
use fail::Failure;
use futures::future;

/// Retrieves monitoring metrics information from server
pub fn metrics<'a>(server_info: ServerInfo, http_client: &'a HttpClient) -> impl Future<Item=Value, Error=Failure> + 'a {
    let request = build_request(
        Method::Get,
        format!("{}/metrics", server_info.url_base).parse().expect("Failed to construct path for HTTP request"),
        server_info.authorization,
        None);
    future::result(request)
        .and_then(move |r| execute_request(http_client, r))
        .and_then(read_full_resp_body_utf8_json)
}
