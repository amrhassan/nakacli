
use hyper;
use hyper::{Method, Request, Uri};
use hyper::header;
use futures::future;
use futures::Future;
use serde_json;
use serde_json::Value;
use futures::Stream;
use fail::{failure, Failure};
use server::Authorization;
use auth;

pub fn build_request(method: Method, uri: Uri, authorization: Authorization, body: Option<&str>) -> Result<Request, Failure> {

    let mut request = Request::new(method, uri);

    let bearer_token = match authorization {
        Authorization::None => None,
        Authorization::BearerToken(token) => Some(token.to_owned()),
        Authorization::Zign => Some(auth::zign()?)
    };

    if let Some(bearer_token_value) = bearer_token {
        request.headers_mut().set(header::Authorization("Bearer".to_owned() + &bearer_token_value))
    }

    if let Some(body_value) = body {
        request.set_body(body_value.to_owned());
    }

    Ok(request)
}

pub fn read_full_resp_body_utf8_json(response: hyper::Response) -> impl Future<Item=Value, Error=Failure> {
    response
        .body()
        .concat2()
        .map_err(|err| failure("HTTP error", err))
        .and_then(|chunk| future::result(String::from_utf8(chunk.into_iter().collect()).map_err(|err| failure("UTF-8 decoding failure", err))))
        .and_then(|text| future::result(serde_json::from_str(&text).map_err(|err| failure("JSON parsing failure", err))))
}
