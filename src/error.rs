//! Errors that might occur while using the RING API.

use std::fmt::{ Display, Formatter, Result as FmtResult };
use std::io::Error as IoError;
use std::error::Error as StdError;
use serde::ser::Error as SerError;
use reqwest::Error as ReqwestError;

/// A RING API error.
#[derive(Debug)]
pub enum Error {
    /// An HTTP error (either a network problem or a RING API error).
    Reqwest(ReqwestError),
    /// A serialization error.
    Serialization(String),
    /// An I/O error.
    Io(IoError),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> FmtResult {
        match *self {
            Error::Reqwest(ref cause) => write!(
                formatter, "RING error: {}", cause
            ),
            Error::Serialization(ref message) => write!(
                formatter, "Serialization error: {}", message
            ),
            Error::Io(ref cause) => write!(
                formatter, "I/O error: {}", cause
            ),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            Error::Reqwest(ref cause) => Some(cause),
            Error::Serialization(_) => None,
            Error::Io(ref cause) => Some(cause),
        }
    }
}

impl From<ReqwestError> for Error {
    fn from(error: ReqwestError) -> Self {
        Error::Reqwest(error)
    }
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        Error::Io(error)
    }
}

impl SerError for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Serialization(msg.to_string())
    }
}

/// A type alias for results that may contain a RING `Error`.
pub type Result<T> = std::result::Result<T, Error>;
