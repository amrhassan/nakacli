use tokio_core::reactor::{Core, Handle};
use hyper::client::{HttpConnector};
use hyper_tls::HttpsConnector;
use hyper::Client;
use std::time::Duration;
use global::GlobalParams;

pub struct Application {
    pub core: Core,
    pub http_client: Client<HttpsConnector<HttpConnector>>,
    pub streaming_http_client: Client<HttpsConnector<HttpConnector>>,
}

const DNS_WORKER_THREADS: usize = 1;

impl Application {

    pub fn new(global_params: &GlobalParams) -> Application {
        let core = Core::new().expect("Failed to initialize HTTP client event loop");
        let http_client = build_http_client(global_params.network_timeout, core.handle());
        let streaming_http_client = build_http_client(Some(Duration::from_secs(24*60*60)), core.handle());
        Application { core, http_client, streaming_http_client }
    }
}

fn build_http_client(network_timeout: Option<Duration>, handle: Handle) -> Client<HttpsConnector<HttpConnector>> {
    Client::configure()
        .keep_alive_timeout(network_timeout)
        .connector(HttpsConnector::new(DNS_WORKER_THREADS, &handle).expect("Failed to initialize TLS for HTTPS"))
        .build(&handle)
}
