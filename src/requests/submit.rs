//! Submit a job.

use std::borrow::Cow;
use reqwest::Method;
use super::Request;
use crate::{
    settings::Settings,
    job::{ JobId, JobStatus },
};

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

/// The response from the "submit" endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmitResponse {
    /// The Job ID which can be used later for querying the results.
    #[serde(rename = "jobid")]
    pub job_id: JobId,
    /// The initially-reported status of the job, usually "in progress".
    pub status: JobStatus,
}

impl Request for SubmitId {
    type Response = SubmitResponse;
    const METHOD: Method = Method::POST;

    fn endpoint(&self) -> Cow<str> {
        Cow::from("/submit")
    }
}
