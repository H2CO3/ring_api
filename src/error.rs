//! Errors that might occur while using the RING API.

use std::fmt::{ Display, Formatter, Result as FmtResult };
use std::error::Error as StdError;
use reqwest::Error as ReqwestError;

/// A RING API error.
#[derive(Debug)]
pub struct Error(ReqwestError);

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> FmtResult {
        write!(formatter, "RING error: {}", self.0)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.0)
    }
}

impl From<ReqwestError> for Error {
    fn from(error: ReqwestError) -> Self {
        Error(error)
    }
}

/// A type alias for results that may contain a RING `Error`.
pub type Result<T> = std::result::Result<T, Error>;
