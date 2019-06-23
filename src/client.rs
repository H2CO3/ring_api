//! A RING HTTP API client.

use serde::Serialize;
use reqwest::{ Client as ReqwestClient, RequestBuilder };
use crate::{
    requests::{ Request, RequestBody },
    error::Result,
    multipart::to_form,
};

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
            .ring_body(request.body())?
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

/// Private trait for extending the client builder so that it can send the body
/// in different formats, decided dynamically.
trait RequestBuilderExt: Sized {
    /// Sends the body in the specified format, or does nothing if it is `None`.
    fn ring_body<T: Serialize>(self, body: RequestBody<T>) -> Result<Self>;
}

impl RequestBuilderExt for RequestBuilder {
    fn ring_body<T: Serialize>(self, body: RequestBody<T>) -> Result<Self> {
        Ok(match body {
            RequestBody::None => self,
            RequestBody::Json(value) => self.json(&value),
            RequestBody::Query(value) => self.query(&value),
            RequestBody::Form(value) => self.form(&value),
            RequestBody::Multipart(value) => self.multipart(to_form(&value)?),
        })
    }
}
