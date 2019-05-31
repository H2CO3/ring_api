//! Submit a job.

use std::borrow::Cow;
use std::collections::HashMap;
use reqwest::Method;
use super::Request;
use crate::settings::Settings;

/// Submitting a RING job based on a known PDB ID.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubmitId {
    /// The PDB ID of the protein to be RING'd.
    #[serde(rename = "pdbName")]
    pub pdb_id: String,
    /// The settings with which to perform the job.
    #[serde(flatten)]
    pub settings: Settings,
}

impl Request for SubmitId {
    type Response = HashMap<String, String>;
    const METHOD: Method = Method::POST;

    fn endpoint(&self) -> Cow<str> {
        Cow::from("/submit")
    }
}
