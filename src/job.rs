//! Types for manipulating jobs.

use std::fmt::{ Display, Formatter, Result as FmtResult };

/// This is received in responses from the RING server.
/// Indicates what phase a specific job is currently in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JobStatus {
    /// The hob is in progress.
    #[serde(rename = "db")]
    InProgress,
    /// The job has completed successfully, results are ready to retrieve.
    #[serde(rename = "complete")]
    Complete,
    /// The job has encountered an error and no results are available.
    #[serde(rename = "error")]
    Failed,
}

/// A RING Job ID.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct JobId(String);

impl JobId {
    /// Returns the string representation of the job ID.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for JobId {
    fn from(string: String) -> Self {
        JobId(string)
    }
}

impl From<JobId> for String {
    fn from(job_id: JobId) -> Self {
        job_id.0
    }
}

impl AsRef<str> for JobId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Display for JobId {
    fn fmt(&self, formatter: &mut Formatter) -> FmtResult {
        self.as_str().fmt(formatter)
    }
}
