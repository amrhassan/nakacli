
use http::*;
use futures::Future;
use hyper::{Method, StatusCode};
use server::ServerInfo;
use fail::Failure;
use futures::future;
use app::Application;
use fail::{die, failureln};
use output::pretty_json;

/// Retrieves monitoring metrics information from server
fn metrics<'a>(server_info: ServerInfo, http_client: &'a HttpClient) -> impl Future<Item=(StatusCode, String), Error=Failure> + 'a {
    let request = build_request(
        Method::Get,
        format!("{}/metrics", server_info.url_base).parse().expect("Failed to construct path for HTTP request"),
        server_info.authorization,
        None);
    future::result(request)
        .and_then(move |r| execute_request(http_client, r))
        .and_then(|resp| { let status = resp.status(); read_full_resp_body_utf8(resp).map(move |v| (status, v))})
}

pub fn run(server_info: ServerInfo, application: &mut Application) {
    let action = metrics(server_info, &application.http_client);
    match application.core.run(action) {
        Ok((StatusCode::Ok, output)) => {
            println!("{}", pretty_json(&output));
        },
        Ok((_, output)) => {
            die(1, failureln("Unexpected response:", pretty_json(&output)));
        }
        Err(err) => {
            die(1, err);
        }
    }
}
