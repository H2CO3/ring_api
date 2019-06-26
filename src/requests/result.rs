//! Retrieving the results of a RING Job

use std::fmt::{ Display, Formatter, Result as FmtResult };
use std::str::FromStr;
use std::borrow::Cow;
use serde::{
    ser::{ Serialize, Serializer },
    de::{ Deserialize, Deserializer, Visitor, Error as DeError },
};
use serde_json::{ self, Value };
use super::Request;
use crate::{
    settings::Settings,
    job::{ JobId, JobStatus },
    error::Error,
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
    /// The unique ID of this node.
    #[serde(rename = "NodeId")]
    pub node_id: NodeId,
    /// The name/ID of the chain this residue belongs in.
    #[serde(rename = "Chain")]
    pub chain_id: char,
    /// The position of the residue inside the sequence, according to PDB.
    /// **NOTE:** this sometimes is 0 or a **negative** integer.
    #[serde(rename = "Position")]
    pub position: isize,
    /// The amino acid that this node represents.
    #[serde(rename = "Residue")]
    pub residue: Residue,
    /// X coordinate.
    pub x: f64,
    /// Y coordinate.
    pub y: f64,
    /// Z coordinate.
    pub z: f64,
    /// Secondary structure, as predicted by DSSP.
    #[serde(rename = "Dssp")]
    pub dssp_structure: DsspStructure,
    /// The degree of, i.e. the number of edges from and to, this node.
    #[serde(rename = "Degree")]
    pub degree: usize,
    /// Relative solvent accessibility (RSA).
    #[serde(rename = "Accessibility")]
    pub accessibility: f64,
    /// The B-factor of the alpha carbon.
    #[serde(rename = "Bfactor_CA")]
    pub bfactor_ca: f64,
    /// The TAP energy.
    #[serde(rename = "Tap", default, skip_serializing_if = "Option::is_none")]
    pub tap_energy: Option<f64>,
    /// The RAPDF energy (calculated based on statistical potentials).
    #[serde(rename = "Rapdf", default, skip_serializing_if = "Option::is_none")]
    pub rapdf_energy: Option<f64>,
    /// Only for letting RINanylezer/StructureViz and Chimera love each other.
    #[serde(rename = "pdbFileName")]
    pub pdb_file_name: PdbFileName,
    /// Shannon entropy computed from a multiple alignment (MSA=true).
    #[serde(rename = "Entropy", default, skip_serializing_if = "Option::is_none")]
    pub entropy: Option<f64>,
    /// Cumulative mutual entropy. Yes, it's incorrectly called "comulative"
    /// in the JSON returned by the API.
    #[serde(rename = "MIcomulative", default, skip_serializing_if = "Option::is_none")]
    pub cumul_mutual_entropy: Option<f64>,
}

/// A structured Node ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId {
    /// The chain ID letter found in the PDB structure.
    pub chain_id: char,
    /// The PDB position index. May be negative.
    pub position: isize,
    /// The PDB insertion code.
    pub insertion_code: char,
    /// The amino acid residue kind.
    pub residue: Residue,
}

impl Display for NodeId {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(
            f, "{}:{}:{}:{}",
            self.chain_id,
            self.position,
            self.insertion_code,
            self.residue,
        )
    }
}

impl FromStr for NodeId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v: Vec<_> = s.split(':').collect();

        if v.len() == 4 {
            Ok(NodeId {
                chain_id:       v[0].parse()?,
                position:       v[1].parse()?,
                insertion_code: v[2].parse()?,
                residue:        v[3].parse()?,
            })
        } else {
            Err(Error::Serialization(String::from(
                "Node ID format must be chain:position:insertion:residue"
            )))
        }
    }
}

impl Serialize for NodeId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'a> Deserialize<'a> for NodeId {
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(NodeIdVisitor)
    }
}

/// Just your regular private node ID visitor type.
#[derive(Debug, Clone, Copy, Default)]
struct NodeIdVisitor;

impl<'a> Visitor<'a> for NodeIdVisitor {
    type Value = NodeId;

    fn expecting(&self, f: &mut Formatter) -> FmtResult {
        f.write_str("a string representation of a node ID")
    }

    fn visit_str<E: DeError>(self, s: &str) -> Result<Self::Value, E> {
        NodeId::from_str(s).map_err(E::custom)
    }

}

/// For now. TODO(H2CO3): make this a `struct`.
pub type PdbFileName = String;

/// An amino acid residue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Residue {
    /// Alanine
    #[serde(rename = "ALA")]
    Alanine,
    /// Arginine
    #[serde(rename = "ARG")]
    Arginine,
    /// Asparagine
    #[serde(rename = "ASN")]
    Asparagine,
    /// AsparticAcid
    #[serde(rename = "ASP")]
    AsparticAcid,
    /// Cysteine
    #[serde(rename = "CYS")]
    Cysteine,
    /// GlutamicAcid
    #[serde(rename = "GLU")]
    GlutamicAcid,
    /// Glutamine
    #[serde(rename = "GLN")]
    Glutamine,
    /// Glycine
    #[serde(rename = "GLY")]
    Glycine,
    /// Homocysteine
    #[serde(rename = "HCY")]
    Homocysteine,
    /// Histidine
    #[serde(rename = "HIS")]
    Histidine,
    /// Homoserine
    #[serde(rename = "HSE")]
    Homoserine,
    /// Isoleucine
    #[serde(rename = "ILE")]
    Isoleucine,
    /// Leucine
    #[serde(rename = "LEU")]
    Leucine,
    /// Lysine
    #[serde(rename = "LYS")]
    Lysine,
    /// Methionine
    #[serde(rename = "MET")]
    Methionine,
    /// Norleucine
    #[serde(rename = "NLE")]
    Norleucine,
    /// Norvaline
    #[serde(rename = "NVA")]
    Norvaline,
    /// Ornithine
    #[serde(rename = "ORN")]
    Ornithine,
    /// Penicillamine
    #[serde(rename = "PEN")]
    Penicillamine,
    /// Phenylalanine
    #[serde(rename = "PHE")]
    Phenylalanine,
    /// Proline
    #[serde(rename = "PRO")]
    Proline,
    /// Pyrrolysine
    #[serde(rename = "PYL")]
    Pyrrolysine,
    /// Selenocysteine
    #[serde(rename = "SEC")]
    Selenocysteine,
    /// Serine
    #[serde(rename = "SER")]
    Serine,
    /// Threonine
    #[serde(rename = "THR")]
    Threonine,
    /// Tryptophan
    #[serde(rename = "TRP")]
    Tryptophan,
    /// Tyrosine
    #[serde(rename = "TYR")]
    Tyrosine,
    /// Valine
    #[serde(rename = "VAL")]
    Valine,
    /// Asparagine or Aspartic Acid
    #[serde(rename = "ASX")]
    AsparagineOrAsparticAcid,
    /// Glutamine or Glutamic Acid
    #[serde(rename = "GLX")]
    GlutamineOrGlutamicAcid,
    /// Leucine or Isoleucine
    #[serde(rename = "XLE")]
    LeucineOrIsoleucine,
    /// Unknown
    #[serde(rename = "XAA")]
    Unknown,
}

impl FromStr for Residue {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(Value::from(s)).map_err(From::from)
    }
}

impl Display for Residue {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match serde_json::to_value(self) {
            Ok(Value::String(ref s)) => f.write_str(s),
            _ => panic!("Residue didn't serialize to a string"),
        }
    }
}

/// Secondary Structure as predicted by DSSP.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DsspStructure {
    /// No structure predicted by DSSP
    #[serde(rename = " ")]
    None,
    /// 3-turn helix or 3-10 helix
    #[serde(rename = "G")]
    Helix310,
    /// 4-turn helix or alpha helix
    #[serde(rename = "H")]
    HelixAlpha,
    /// 5-turn helix or pi-helix
    #[serde(rename = "I")]
    HelixPi,
    /// Hydrogen bonded turn
    #[serde(rename = "T")]
    TurnHBond,
    /// Extended strand in parallel and/or anti-parallel beta sheet conformation
    #[serde(rename = "E")]
    BetaExtended,
    /// Residue in isolated beta-bridge
    #[serde(rename = "B")]
    BetaIsolated,
    /// Bend
    #[serde(rename = "S")]
    Bend,
}

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
