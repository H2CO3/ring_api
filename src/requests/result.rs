//! Retrieving the results of a RING Job

use std::borrow::Cow;
use super::Request;
use crate::{
    settings::Settings,
    job::{ JobId, JobStatus },
};

/// Request the result of a job
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RetrieveResult {
    /// The RING Job ID for which to retrieve the results.
    pub job_id: JobId,
}

/// Response containing the result for a completed RING job.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetrieveResultResponse {
    /// The job ID is echoed back.
    #[serde(rename = "_id")]
    pub job_id: JobId,
    /// The current status of the job.
    pub status: JobStatus,
    /// If a PDB ID has been submitted, it is captured here.
    #[serde(default, rename = "pdbName", skip_serializing_if = "Option::is_none")]
    pub pdb_id: Option<String>,
    /// If a file name has been supplied upon submission, it is captured here.
    #[serde(default, rename = "fileName", skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    /// The job settings are echoed back.
    #[serde(flatten, default)]
    pub settings: Settings,
    /// Nodes of the interaction graph.
    pub nodes: Vec<Node>,
    /// Edges of the interaction graph.
    pub edges: Vec<Edge>,
}

/// A node in the interaction graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    /// The amino acid that this node represents.
    #[serde(rename = "Residue")]
    pub residue: Residue,
    /// The B-factor of the alpha carbon.
    #[serde(rename = "Bfactor_CA")]
    pub bfactor_ca: f64,
    /// The TAP energy.
    #[serde(rename = "Tap", default, skip_serializing_if = "Option::is_none")]
    pub tap_energy: Option<f64>,
    /// The degree of, i.e. the number of edges from and to, this node.
    #[serde(rename = "Degree")]
    pub degree: usize,
    /// Relative solvent accessibility (RSA).
    #[serde(rename = "Accessibility")]
    pub accessibility: f64,
    /// The RAPDF energy (calculated based on statistical potentials).
    #[serde(rename = "Rapdf", default, skip_serializing_if = "Option::is_none")]
    pub rapdf_energy: Option<f64>,
    /// Only for letting RINanylezer/StructureViz and Chimera love each other.
    #[serde(rename = "pdbFileName")]
    pub pdb_file_name: PdbFileName,
    /// The unique ID of this node.
    #[serde(rename = "NodeId")]
    pub node_id: NodeId,
    /// The position of the residue inside the sequence, according to PDB.
    /// **NOTE:** this sometimes is 0 or a **negative** integer.
    #[serde(rename = "Position")]
    pub position: isize,
    /// The name/ID of the chain this residue belongs in.
    #[serde(rename = "Chain")]
    pub chain_id: String,
    /// X position.
    pub x: f64,
    /// Y position.
    pub y: f64,
    /// Z position.
    pub z: f64,
    /// Secondary structure, as predicted by DSSP.
    #[serde(rename = "Dssp")]
    pub dssp_structure: DsspStructure,
    /// Shannon entropy computed from a multiple alignment (MSA=true).
    #[serde(rename = "Entropy", default, skip_serializing_if = "Option::is_none")]
    pub entropy: Option<f64>,
    /// Cumulative mutual entropy. Yes, it's incorrectly called "comulative"
    /// in the JSON returned by the API.
    #[serde(rename = "MIcomulative", default, skip_serializing_if = "Option::is_none")]
    pub cumul_mutual_entropy: Option<f64>,
}

/// For now. TODO(H2CO3): make this an `enum`.
pub type Residue = String;
/// For now. TODO(H2CO3): make this a `struct`.
pub type NodeId = String;
/// For now. TODO(H2CO3): make this a `struct`.
pub type PdbFileName = String;
/// For now. TODO(H2CO3): make this an `enum`.
pub type DsspStructure = String;

/// An edge in the interaction graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Edge {}

impl Request for RetrieveResult {
    type Body = ();
    type Response = RetrieveResultResponse;

    fn endpoint(&self) -> Cow<str> {
        format!("/results/{}?engine=d3", self.job_id).into()
    }
}
