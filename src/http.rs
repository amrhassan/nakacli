
use hyper;
use hyper::{Method, Request};
use hyper::header;
use futures::future;
use futures::Future;
use futures::Stream;
use output::{failure, Failure};
use server::Authorization;
use auth;
use hyper::client::{HttpConnector};
use hyper_tls::HttpsConnector;
use hyper::{Response, Client, StatusCode};
use server::ServerInfo;
use serde_json::{Value};
use serde_json;

pub type HttpClient = Client<HttpsConnector<HttpConnector>>;

/// Builds a request from the given parameters. If bod is provided, the Content-Type header is set to `application/json`.
pub fn build_request(method: Method, path: &str, server_info: &ServerInfo, body: Option<&Value>) -> Result<Request, Failure> {

    let uri = format!("{}{}", server_info.url_base, path).parse().expect("Failed to construct URI for HTTP request");
    let mut request = Request::new(method, uri);

    let bearer_token = match server_info.authorization {
        Authorization::None => None,
        Authorization::BearerToken(token) => Some(token.to_owned()),
        Authorization::Zign => Some(auth::zign()?)
    };

    if let Some(bearer_token_value) = bearer_token {
        request.headers_mut().set(header::Authorization(header::Bearer { token: bearer_token_value }));
    }

    if let Some(body_value) = body {
        let text_body = serde_json::to_string(body_value).map_err(|err| failure("Failed to JSON-serialize the request body", err))?;
        request.headers_mut().set(header::ContentType::json());
        request.set_body(text_body);
    }

    Ok(request)
}

/// Accumulates a response body fully into memory.
///
/// WARNING: Could potentially consume all available memory if response body is big enough.
pub fn read_full_resp_body_utf8(response: hyper::Response) -> impl Future<Item=String, Error=Failure> {
    response
        .body()
        .concat2()
        .map_err(|err| failure("HTTP error", err))
        .and_then(|chunk| String::from_utf8(chunk.into_iter().collect()).map_err(|err| failure("UTF-8 decoding failure", err)))
}

/// Executes a `Request` yielding a `Response`
pub fn execute_request(http_client: &HttpClient, request: Request) -> impl Future<Item=Response, Error=Failure> {
    http_client.request(request).map_err(|err| failure("Sending HTTP request failed", err))
}

/// Executes an HTTP request with the given paramters, and returns the [[StatusCode]] and full body of the response
pub fn execute_and_read_full_resp_body_utf8<'a>(
    http_client: &'a HttpClient,
    method: Method,
    path: &'a str,
    server_info: &'a ServerInfo,
    body: Option<&Value>) -> impl Future<Item=(StatusCode, String), Error=Failure> + 'a {
    let request = build_request(method, path, server_info, body);
    future::result(request)
        .and_then(move |r| execute_request(http_client, r))
        .and_then(|resp| { let status = resp.status(); read_full_resp_body_utf8(resp).map(move |v| (status, v))})
}

#[cfg(test)]
mod tests {

    use super::*;
    use hyper::header;
    use std::str::FromStr;

    #[test]
    fn builds_request() {

        // Arbitraries
        let arb_method = Method::Put;
        let arb_path = "/fjwlfkjewg/wgwe";
        let arb_authorization = Authorization::BearerToken("sup");
        let arb_server_info = ServerInfo { url_base: "http://hostname", authorization: arb_authorization };
        let arb_body = json!({"name": "John Doe", "age": 43, "phones": [ "+44 1234567", "+44 2345678" ] });


        let json_request = build_request(arb_method.clone(), arb_path, &arb_server_info, Some(&arb_body)).ok().expect("Failed to construct request");

        assert_eq!(arb_path, json_request.path());
        assert_eq!(&arb_method, json_request.method());
        assert_eq!(Some(&header::ContentType::json()), json_request.headers().get::<header::ContentType>());

        match arb_authorization {
            Authorization::None => assert!(!json_request.headers().has::<header::Authorization<header::Bearer>>()),
            Authorization::BearerToken(token_value) => {
                let expected_header = header::Authorization(header::Bearer::from_str(token_value).expect("Failed to construct expected header"));
                let existing_header = json_request.headers().get::<header::Authorization<header::Bearer>>();
                assert_eq!(Some(&expected_header), existing_header)
            },
            _ => assert!(false),
        }

        let request_body = {
            let bytes: Vec<u8> = json_request.body().concat2().wait().expect("Failed to read request body").into_iter().collect();
            let text: String = String::from_utf8(bytes).expect("Failed to UTF-8 decode request body");
            let json: Value = serde_json::from_str(&text).expect("Failed to JSON-decode body");
            json
        };
        assert_eq!(arb_body, request_body)
    }
}
