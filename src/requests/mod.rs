//! Interface to various endpoints of the RING HTTP API.

use std::borrow::Cow;
use serde::{ Serialize, Deserialize };
use reqwest::{ Method, header::HeaderMap };
pub use submit::*;
pub use status::*;

pub mod submit;
pub mod status;

/// What body, if any, should be sent with a request?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RequestBody<T> {
    /// Do not send body or modify URL query parameters at all.
    None,
    /// Serialize value as JSON and send it in body.
    Json(T),
    /// Serialize value as query string and append it to URL query parameters.
    Query(T),
    /// Serialize value as x-www-form-urlencoded and send it in body.
    Form(T),
    /// Serialize value as multipart/form-data and send it in body.
    Multipart(T),
}

impl<T> Default for RequestBody<T> {
    fn default() -> Self {
        RequestBody::None
    }
}

/// A RING API request.
pub trait Request {
    /// The type of the body for this request. TODO(H2CO3): default to `()`.
    type Body: Serialize;

    /// The "return type" of the request.
    type Response: for<'de> Deserialize<'de>;

    /// The HTTP method ("verb") for the request.
    const METHOD: Method = Method::GET;

    /// The endpoint: the part of the URL/path that follows the base URL.
    fn endpoint(&self) -> Cow<str>;

    /// Additional headers for this request.
    fn headers(&self) -> HeaderMap {
        Default::default()
    }

    /// The body of the request, if any
    fn body(&self) -> RequestBody<&Self::Body> {
        Default::default()
    }
}

impl<R: Request> Request for &R {
    type Body = R::Body;
    type Response = R::Response;

    const METHOD: Method = R::METHOD;

    fn endpoint(&self) -> Cow<str> {
        (**self).endpoint()
    }

    fn headers(&self) -> HeaderMap {
        (**self).headers()
    }

    fn body(&self) -> RequestBody<&Self::Body> {
        (**self).body()
    }
}

impl<R: Request> Request for &mut R {
    type Body = R::Body;
    type Response = R::Response;

    const METHOD: Method = R::METHOD;

    fn endpoint(&self) -> Cow<str> {
        (**self).endpoint()
    }

    fn headers(&self) -> HeaderMap {
        (**self).headers()
    }

    fn body(&self) -> RequestBody<&Self::Body> {
        (**self).body()
    }
}
