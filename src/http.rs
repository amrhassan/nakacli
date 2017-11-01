
use hyper;
use hyper::{Method, Request, Uri};
use hyper::header;
use futures::future;
use futures::Future;
use serde_json;
use serde_json::Value;
use futures::Stream;
use fail::failure;

pub fn build_request(method: Method, uri: Uri, oauth2_token: Option<&str>, body: Option<&str>) -> Request {
    let mut request = Request::new(method, uri);
    if let Some(token_value) = oauth2_token {
        request.headers_mut().set(header::Authorization("Bearer".to_owned() + token_value))
    }
    if let Some(body_value) = body {
        request.set_body(body_value.to_owned());
    }
    request
}

pub fn read_full_resp_body_utf8_json(response: hyper::Response) -> impl Future<Item=Value, Error=String> {
    response
        .body()
        .concat2()
        .map_err(|err| failure("HTTP error", err))
        .and_then(|chunk| future::result(String::from_utf8(chunk.into_iter().collect()).map_err(|err| failure("UTF-8 decoding failure", err))))
        .and_then(|text| future::result(serde_json::from_str(&text).map_err(|err| failure("JSON parsing failure", err))))
}