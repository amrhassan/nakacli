use tokio_core::reactor::Core;
use hyper::client::{HttpConnector};
use hyper_tls::HttpsConnector;
use hyper::Client;
use std::time::Duration;

pub struct Application {
    pub core: Core,
    pub http_client: Client<HttpsConnector<HttpConnector>>
}

const DNS_WORKER_THREADS: usize = 1;

impl Application {

    pub fn new() -> Application {
        let core = Core::new().expect("Failed to initialize HTTP client event loop");
        let handle = core.handle();
        let http_client = Client::configure()
            .keep_alive_timeout(Some(Duration::from_secs(24*60*60)))
            .connector(HttpsConnector::new(DNS_WORKER_THREADS, &handle).expect("Failed to initialize TLS for HTTPS"))
            .build(&handle);
        Application { core, http_client }
    }
}
