//! Interface to various endpoints of the RING HTTP API.

use std::borrow::Cow;
use serde::{ Serialize, Deserialize };
use reqwest::{ Method, header::HeaderMap };
pub use submit::*;
pub use status::*;

pub mod submit;
pub mod status;

/// A RING API request.
pub trait Request: Serialize {
    /// The "return type" of the request.
    type Response: for<'de> Deserialize<'de>;

    /// The HTTP method ("verb") for the request.
    const METHOD: Method;

    /// The endpoint: the part of the URL/path that follows the base URL.
    fn endpoint(&self) -> Cow<str>;

    /// Additional headers for this request.
    fn headers(&self) -> HeaderMap {
        HeaderMap::new()
    }
}

impl<R: Request> Request for &R {
    type Response = R::Response;

    const METHOD: Method = R::METHOD;

    fn endpoint(&self) -> Cow<str> {
        (**self).endpoint()
    }

    fn headers(&self) -> HeaderMap {
        (**self).headers()
    }
}

impl<R: Request> Request for &mut R {
    type Response = R::Response;

    const METHOD: Method = R::METHOD;

    fn endpoint(&self) -> Cow<str> {
        (**self).endpoint()
    }

    fn headers(&self) -> HeaderMap {
        (**self).headers()
    }
}
