use tokio_core::reactor::Core;
use hyper::client::{HttpConnector};
use hyper_tls::HttpsConnector;
use hyper::{Request, Response, Client};
use futures::Future;
use fail::failure;

pub struct Application {
    core: Core,
    http_client: Client<HttpsConnector<HttpConnector>>
}

const DNS_WORKER_THREADS: usize = 1;

impl Application {

    pub fn new() -> Application {
        let core = Core::new().expect("Failed to initialize HTTP client event loop");
        let handle = core.handle();
        let http_client = Client::configure()
            .connector(HttpsConnector::new(DNS_WORKER_THREADS, &handle).expect("Failed to initialize TLS for HTTPS"))
            .build(&handle);
        Application { core, http_client }
    }

    /// Returns the action of executing the given HTTP request yielding an HTTP response
    pub fn execute_request(&self, request: Request) -> impl Future<Item=Response, Error=String> {
        self.http_client
            .request(request)
            .map_err(|err| failure("Sending HTTP request failed", err))
    }

    /// Runs the given future on the main event loop
    pub fn run<A, F: Future<Item=A, Error=String>>(&mut self, future: F) -> Result<A, String> {
        self.core.run(future)
    }
}
