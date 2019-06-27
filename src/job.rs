//! Types for manipulating jobs.

use std::fmt::{ Display, Formatter, Result as FmtResult };

/// This is received in responses from the RING server.
/// Indicates what phase a specific job is currently in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JobStatus {
    /// The hob is in progress.
    #[serde(rename = "db")]
    InProgress,
    /// Part of the job has been completed and some of the results are available.
    /// This is typically returned when the job involves performing an MSA,
    /// and the rest of the computation is done but PSIBLAST is still running.
    #[serde(rename = "partial")]
    Partial,
    /// The job has completed successfully, results are ready to retrieve.
    #[serde(rename = "complete")]
    Complete,
    /// The job has encountered an error and no results are available.
    #[serde(rename = "error")]
    Failed,
}

impl Display for JobStatus {
    fn fmt(&self, formatter: &mut Formatter) -> FmtResult {
        formatter.pad(match *self {
            JobStatus::InProgress => "in progress",
            JobStatus::Partial    => "partial",
            JobStatus::Complete   => "complete",
            JobStatus::Failed     => "failed",
        })
    }
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

impl From<&str> for JobId {
    fn from(string: &str) -> Self {
        JobId(string.into())
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
        formatter.pad(self.as_str())
    }
}
