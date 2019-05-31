//! A RING HTTP API client.

use reqwest::Client as ReqwestClient;
use crate::requests::Request;
use crate::error::Result;

/// The base URL for the RING API.
static BASE_URL: &str = "http://protein.bio.unipd.it/ringws";

/// The main entry point to the RING webservice.
#[derive(Debug, Clone)]
pub struct Client {
    /// The backing HTTP client.
    client: ReqwestClient,
}

impl Client {
    /// Creates a RING web client.
    pub fn new() -> Self {
        Client {
            client: ReqwestClient::new()
        }
    }

    /// Sending requests.
    pub fn send<R: Request>(&self, request: R) -> Result<R::Response> {
        let endpoint = request.endpoint();
        let endpoint = endpoint.trim_matches('/');
        let url = format!("{}/{}", BASE_URL, endpoint);

        self.client
            .request(R::METHOD, &url)
            .headers(request.headers())
            .json(&request)
            .send()
            .and_then(|mut resp| resp.json())
            .map_err(From::from)
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
