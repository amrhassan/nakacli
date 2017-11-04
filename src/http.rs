
use hyper;
use hyper::{Method, Request, Uri};
use hyper::header;
use futures::future;
use futures::Future;
use futures::Stream;
use fail::{failure, Failure};
use server::Authorization;
use auth;
use hyper::client::{HttpConnector};
use hyper_tls::HttpsConnector;
use hyper::{Response, Client};

pub type HttpClient = Client<HttpsConnector<HttpConnector>>;

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

pub fn read_full_resp_body_utf8(response: hyper::Response) -> impl Future<Item=String, Error=Failure> {
    response
        .body()
        .concat2()
        .map_err(|err| failure("HTTP error", err))
        .and_then(|chunk| future::result(String::from_utf8(chunk.into_iter().collect()).map_err(|err| failure("UTF-8 decoding failure", err))))
}

pub fn execute_request(http_client: &HttpClient, request: Request) -> impl Future<Item=Response, Error=Failure> {
    http_client.request(request).map_err(|err| failure("Sending HTTP request failed", err))
}
