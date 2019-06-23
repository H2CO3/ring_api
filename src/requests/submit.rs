//! Submit a job.

use std::borrow::Cow;
use std::io::Read;
use std::path::Path;
use std::ffi::OsStr;
use std::fs::read_to_string;
use lazy_static::lazy_static;
use reqwest::Method;
use regex::Regex;
use super::{ Request, RequestBody };
use crate::{
    settings::Settings,
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
    pub pdb_structure: String,
    /// The file name, if any (optional).
    #[serde(rename = "fileName", default, skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    /// The PDB ID, if any (optional).
    #[serde(rename = "pdbName", default, skip_serializing_if = "Option::is_none")]
    pub pdb_id: Option<String>,
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
    /// default settings, and no file name. It tries to guess the PDB ID
    /// from the contents of the structure.
    pub fn with_pdb_structure<T: Into<String>>(pdb_structure: T) -> Self {
        let structure = pdb_structure.into();
        let maybe_pdb_id = pdb_id_from_header(&structure);

        SubmitStructure {
            pdb_structure: structure,
            file_name: None,
            pdb_id: maybe_pdb_id,
            settings: Settings::default(),
        }
    }

    /// Convenience constructor.
    /// Creates a submit structure request from an `io::Read`, with the
    /// default settings and no file name. It tries to guess the PDB ID
    /// from the contents of the structure.
    pub fn with_reader<R: Read>(mut reader: R) -> Result<Self> {
        let mut structure = String::new();
        reader.read_to_string(&mut structure)?;
        Ok(Self::with_pdb_structure(structure))
    }

    /// Convenience constructor.
    /// Creates a submit structure request from a file, with the
    /// default settings and the specified file name (if the latter can be
    /// converted to a UTF-8 string).
    /// It tries to guess the PDB ID from the structure first. If that fails,
    /// it also attempts to parse the **file name** for a PDB ID.
    pub fn with_pdb_file<P: AsRef<Path>>(file: P) -> Result<Self> {
        let path = file.as_ref();
        let maybe_file_name = path
            .file_name()
            .and_then(OsStr::to_str);
        let structure = read_to_string(path)?;
        let maybe_pdb_id = pdb_id_from_header(&structure).or_else(
            || maybe_file_name.and_then(pdb_id_from_file_name)
        );

        Ok(SubmitStructure {
            pdb_structure: structure,
            file_name: maybe_file_name.map(Into::into),
            pdb_id: maybe_pdb_id,
            settings: Settings::default(),
        })
    }

    /// Builder method for unconditionally setting the file name.
    pub fn file_name<T: Into<String>>(self, file_name: T) -> Self {
        SubmitStructure {
            file_name: Some(file_name.into()),
            ..self
        }
    }

    /// Builder method for setting the file name only if it does not exist.
    pub fn or_file_name<T: Into<String>>(self, file_name: T) -> Self {
        if self.file_name.is_none() {
            self.file_name(file_name)
        } else {
            self
        }
    }

    /// Builder method for unconditionally setting the PDB ID.
    pub fn pdb_id<T: Into<String>>(self, pdb_id: T) -> Self {
        SubmitStructure {
            pdb_id: Some(pdb_id.into()),
            ..self
        }
    }

    /// Builder method for setting the PDB ID only if it does not exist.
    pub fn or_pdb_id<T: Into<String>>(self, pdb_id: T) -> Self {
        if self.pdb_id.is_none() {
            self.pdb_id(pdb_id)
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

/// Private helper function for heuristically extracting the PDB ID from the
/// structure header.
fn pdb_id_from_header(header: &str) -> Option<String> {
    lazy_static!{
        static ref PDB_ID_REGEX: Regex = Regex::new(
            r"(?im)^\s*HEADER\s+.*(?P<pdbid>[1-9][0-9A-Z]{3})\s*$"
        ).expect(
            "can't compile PDB ID header regex"
        );
    }

    PDB_ID_REGEX
        .captures(header)?
        .name("pdbid")
        .map(|m| m.as_str().to_lowercase())
}

/// Private helper function for heuristically extracting the PDB ID from the
/// file path.
fn pdb_id_from_file_name(file_name: &str) -> Option<String> {
    lazy_static!{
        static ref PDB_ID_REGEX: Regex = Regex::new(
            r"(?i)^\s*(?:pdb)?(?P<pdbid>[1-9][0-9A-Z]{3})(?:\.(?:ent|pdb|cif))?(?:\.gz)?\s*$"
        ).expect(
            "can't compile PDB ID filename regex"
        );
    }

    PDB_ID_REGEX.captures(file_name)?
        .name("pdbid")
        .as_ref()
        .map(|m| m.as_str().to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pdb_id_from_header_works() {
        let test_cases = &[
            (
                "HEADER    COMPLEX (TRANSFERASE/CYCLIN/INHIBITOR)  03-JUL-96   1JSU     ",
                Some(String::from("1jsu"))
            ),
            (
                "header    complex (transferase/cyclin/inhibitor)  03-jul-96   1jsu     ",
                Some(String::from("1jsu"))
            ),
            (
                "TITLE     P27(KIP1)/CYCLIN A/CDK2 COMPLEX
\nHEADER    COMPLEX (TRANSFERASE/CYCLIN/INHIBITOR)  03-JUL-96   1JSU              ",
                Some(String::from("1jsu"))
            ),
            (
                "FOO BAR BAZ",
                None
            ),
        ];

        for &(structure, ref id) in test_cases {
            assert_eq!(pdb_id_from_header(structure), *id);
        }
    }

    #[test]
    fn pdb_id_from_file_works() {
        let test_cases = &[
            (
                "pdb1jsu.ent.gz",
                Some(String::from("1jsu"))
            ),
            (
                "pdb1jsu.gz.ent",
                None
            ),
            (
                "PDB6s3q",
                Some(String::from("6s3q"))
            ),
            (
                "9MW8",
                Some(String::from("9mw8"))
            ),
            (
                "26x7.pdb",
                Some(String::from("26x7"))
            ),
            (
                "lol6s3q.pdb",
                None
            ),
            (
                "26x78.pdb",
                None
            ),
        ];

        for &(structure, ref id) in test_cases {
            assert_eq!(pdb_id_from_file_name(structure), *id);
        }
    }
}
