//! Submit a job.

use std::borrow::Cow;
use std::io::Read;
use std::path::Path;
use std::ffi::OsStr;
use std::fs::read_to_string;
use reqwest::Method;
use super::{ Request, RequestBody };
use crate::{
    settings::Settings,
    multipart::FormFile,
    job::{ JobId, JobStatus },
    error::Result,
};

/// Submitting a RING job based on a known PDB ID.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubmitId {
    /// The PDB ID of the protein to be RING'd.
    #[serde(rename = "pdbName")]
    pub pdb_id: String,
    /// The settings with which to perform the job.
    #[serde(flatten, default)]
    pub settings: Settings,
}

/// Submitting a request based on a PDB structure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubmitStructure {
    /// The contents of the PDB structure itself. Typically comes from a file.
    #[serde(rename = "file")]
    pub pdb_structure: FormFile,
    /// The file name, if any (optional).
    #[serde(rename = "fileName", default, skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    /// The RING settings.
    #[serde(flatten, default)]
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

impl SubmitId {
    /// Convenience constructor.
    /// Creates a submit ID request with the default settings.
    pub fn with_pdb_id<T: Into<String>>(pdb_id: T) -> Self {
        SubmitId {
            pdb_id: pdb_id.into(),
            settings: Settings::default(),
        }
    }
}

impl SubmitStructure {
    /// Convenience constructor.
    /// Creates a submit structure request with the specified structure,
    /// default settings, and no file name. (Only for the form's
    /// Content-Disposition, a dummy file name will be used.)
    pub fn with_pdb_structure<T: Into<String>>(pdb_structure: T) -> Self {
        SubmitStructure {
            pdb_structure: FormFile::with_contents_and_file_name(
                pdb_structure.into(),
                String::from("rust_ring_api_dummy.pdb"),
            ),
            file_name: None,
            settings: Settings::default(),
        }
    }

    /// Convenience constructor.
    /// Creates a submit structure request from an `io::Read`, with the
    /// default settings and no file name. (Only for the form's
    /// Content-Disposition, a dummy file name will be used.)
    pub fn with_reader<R: Read>(mut reader: R) -> Result<Self> {
        let mut structure = String::new();
        reader.read_to_string(&mut structure)?;
        Ok(Self::with_pdb_structure(structure))
    }

    /// Convenience constructor.
    /// Creates a submit structure request from a file, with the
    /// default settings and the specified file name (if the latter can be
    /// converted to a UTF-8 string - otherwise, a dummy one will be used).
    pub fn with_pdb_file<P: AsRef<Path>>(file: P) -> Result<Self> {
        let path = file.as_ref();
        let maybe_file_name = path
            .file_name()
            .and_then(OsStr::to_str)
            .map(Into::into);
        let structure = read_to_string(path)?;

        Ok(SubmitStructure {
            pdb_structure: FormFile::with_contents_and_file_name(
                structure,
                maybe_file_name.clone().unwrap_or_else(
                    || String::from("rust_ring_api_dummy.pdb")
                )
            ),
            file_name: maybe_file_name,
            settings: Settings::default(),
        })
    }

    /// Builder method for unconditionally setting the file name.
    pub fn file_name<T: Into<String>>(mut self, file_name: T) -> Self {
        let file_name = file_name.into();

        self.file_name.replace(file_name.clone());
        self.pdb_structure.set_file_name(file_name);
        self
    }

    /// Builder method for setting the file name only if it does not exist.
    pub fn or_file_name<T: Into<String>>(self, file_name: T) -> Self {
        if self.file_name.is_none() {
            self.file_name(file_name)
        } else {
            self
        }
    }

    /// Builder method for changing the settings.
    pub fn settings(self, settings: Settings) -> Self {
        SubmitStructure { settings, ..self }
    }
}

impl Request for SubmitId {
    type Body = Self;
    type Response = SubmitResponse;

    const METHOD: Method = Method::POST;

    fn endpoint(&self) -> Cow<str> {
        Cow::from("/submit")
    }

    fn body(&self) -> RequestBody<&Self::Body> {
        RequestBody::Json(self)
    }
}

impl Request for SubmitStructure {
    type Body = Self;
    type Response = SubmitResponse;

    const METHOD: Method = Method::POST;

    fn endpoint(&self) -> Cow<str> {
        Cow::from("/submit")
    }

    fn body(&self) -> RequestBody<&Self::Body> {
        RequestBody::Multipart(self)
    }
}
