//! Endpoint for querying the status of a job.

use std::borrow::Cow;
use reqwest::Method;
use super::Request;
use crate::{
    settings::Settings,
    job::{ JobId, JobStatus },
};

/// A status request.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Status {
    /// The ID of the job we are requesting the status for.
    pub job_id: JobId,
}

/// Response to a status request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatusResponse {
    /// The job ID is echoed back.
    #[serde(rename = "_id")]
    job_id: JobId,
    /// The current status of the job.
    status: JobStatus,
    /// If a PDB ID has been submitted, it is captured here.
    #[serde(default, rename = "pdbName")]
    pdb_id: Option<String>,
    /// If a file name has been supplied upon submission, it is captured here.
    #[serde(default, rename = "fileName")]
    file_name: Option<String>,
    /// The job settings are echoed back.
    #[serde(flatten, default)]
    settings: Settings,
}

impl Request for Status {
    type Response = StatusResponse;
    const METHOD: Method = Method::GET;

    fn endpoint(&self) -> Cow<str> {
        format!("/status/{}", self.job_id).into()
    }
}
