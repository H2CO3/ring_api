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

impl Request for RetrieveResult {
    type Body = ();
    type Response = RetrieveResultResponse;

    fn endpoint(&self) -> Cow<str> {
        format!("/results/{}?engine=d3", self.job_id).into()
    }
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
    pub pdb_file_name: String,
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

impl Display for Residue {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match serde_json::to_value(self) {
            Ok(Value::String(ref s)) => f.pad(s),
            _ => panic!("Residue didn't serialize to a string"),
        }
    }
}

impl FromStr for Residue {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(Value::from(s)).map_err(From::from)
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    /// ID of one of the nodes connected by this edge.
    #[serde(rename = "NodeId1")]
    pub node_id_1: NodeId,
    /// ID of the other node connected by this edge.
    #[serde(rename = "NodeId2")]
    pub node_id_2: NodeId,
    /// The interaction type corresponding to this edge.
    #[serde(rename = "Interaction")]
    pub interaction: Interaction,
    /// The interacting atom in node 1.
    #[serde(rename = "Atom1")]
    pub atom_1: Atom,
    /// The interacting atom in node 2.
    #[serde(rename = "Atom2")]
    pub atom_2: Atom,
    /// The distance in Angstrom between atom centers / mass centers / barycenters
    /// depending on the type of interaction and the type of residue.
    #[serde(rename = "Distance")]
    pub distance: f64,
    /// The angle in degree.
    #[serde(rename = "Angle", with = "serde_angle")]
    pub angle: Option<f64>,
    /// The average bond free energy in KJ/mol according to literature.
    #[serde(rename = "Energy")]
    pub energy: f64,
    /// The donor in a hydrogen bond.
    #[serde(
        rename = "Donor", default,
        deserialize_with = "deserialize_empty_nodeid",
        skip_serializing_if = "Option::is_none",
    )]
    pub donor: Option<NodeId>,
    /// The positive side of an ionic bond.
    #[serde(
        rename = "Positive", default,
        deserialize_with = "deserialize_empty_nodeid",
        skip_serializing_if = "Option::is_none",
    )]
    pub positive: Option<NodeId>,
    /// The cation in a pi-cation interaction.
    #[serde(
        rename = "Cation", default,
        deserialize_with = "deserialize_empty_nodeid",
        skip_serializing_if = "Option::is_none",
    )]
    pub cation: Option<NodeId>,
    /// Mutual Information
    #[serde(rename = "MI", default, skip_serializing_if = "Option::is_none")]
    pub mutual_inf: Option<f64>,
    /// Average Product Correction
    #[serde(rename = "APC", default, skip_serializing_if = "Option::is_none")]
    pub apc: Option<f64>,
    /// Corrected Mutual Information
    #[serde(rename = "MIcorrected", default, skip_serializing_if = "Option::is_none")]
    pub corrected_mi: Option<f64>,
}

/// Descriptor of an Interaction Type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Interaction {
    /// Main interaction type.
    pub main_type: InteractionMainType,
    /// Node 1 subtype.
    pub subtype_1: InteractionSubType,
    /// Node 2 subtype.
    pub subtype_2: InteractionSubType,
}

impl Display for Interaction {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}:{}_{}", self.main_type, self.subtype_1, self.subtype_2)
    }
}

impl FromStr for Interaction {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = s.split(':').collect();

        if parts.len() == 2 {
            let main = parts[0];
            let parts: Vec<_> = parts[1].split('_').collect();

            if parts.len() == 2 {
                Ok(Interaction {
                    main_type: main.parse()?,
                    subtype_1: parts[0].parse()?,
                    subtype_2: parts[1].parse()?,
                })
            } else {
                Err(Error::Serialization(String::from(
                    "Interaction type format must be main:sub1_sub2"
                )))
            }
        } else {
            Err(Error::Serialization(String::from(
                "Interaction type format must be main:sub1_sub2"
            )))
        }
    }
}

impl Serialize for Interaction {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'a> Deserialize<'a> for Interaction {
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(InteractionVisitor)
    }
}

/// A serde visitor for deserializing an `Interaction`.
#[derive(Debug, Clone, Copy, Default)]
struct InteractionVisitor;

impl<'a> Visitor<'a> for InteractionVisitor {
    type Value = Interaction;

    fn expecting(&self, f: &mut Formatter) -> FmtResult {
        f.write_str("an interaction specification string")
    }

    fn visit_str<E: DeError>(self, s: &str) -> Result<Self::Value, E> {
        Interaction::from_str(s).map_err(E::custom)
    }
}

/// The set of possible main interaction types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InteractionMainType {
    /// Hydrogen bond.
    #[serde(rename = "HBOND")]
    HydrogenBond,
    /// van der Waals-interaction.
    #[serde(rename = "VDW")]
    VanDerWaals,
    /// Disulphide bond.
    #[serde(rename = "SSBOND")]
    Disulphide,
    /// Ionic bond.
    #[serde(rename = "IONIC")]
    Ionic,
    /// Pi-pi stacking.
    #[serde(rename = "PIPISTACK")]
    PiPiStack,
    /// Pi-cation interaction.
    #[serde(rename = "PICATION")]
    PiCation,
}

impl Display for InteractionMainType {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match serde_json::to_value(self) {
            Ok(Value::String(ref s)) => f.pad(s),
            _ => panic!("InteractionMainType didn't serialize to a string"),
        }
    }
}

impl FromStr for InteractionMainType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(Value::from(s)).map_err(From::from)
    }
}

impl Display for InteractionSubType {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match serde_json::to_value(self) {
            Ok(Value::String(ref s)) => f.pad(s),
            _ => panic!("InteractionSubType didn't serialize to a string"),
        }
    }
}

impl FromStr for InteractionSubType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(Value::from(s)).map_err(From::from)
    }
}

/// The set of possible interaction subtypes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InteractionSubType {
    /// Interaction on the main chain.
    #[serde(rename = "MC")]
    MainChain,
    /// Interaction on a side chain.
    #[serde(rename = "SC")]
    SideChain,
    /// Interaction on a ligand.
    #[serde(rename = "LIG")]
    Ligand,
}

/// Describes an atom either by its name or by its coordinates.
#[derive(Debug, Clone, PartialEq)]
pub enum Atom {
    /// A named atom.
    Name(String),
    /// A 3D coordinate which can locate an atom, a center of mass, a barycenter, etc.
    Coords {
        /// The X component of the coordinate.
        x: f64,
        /// The Y component of the coordinate.
        y: f64,
        /// The Z component of the coordinate.
        z: f64,
    },
}

impl Display for Atom {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            Atom::Name(ref name) => f.pad(name),
            Atom::Coords { x, y, z } => write!(f, "{},{},{}", x, y, z),
        }
    }
}

impl FromStr for Atom {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = s.split(',').collect();

        if parts.len() == 3 {
            Ok(Atom::Coords {
                x: parts[0].parse()?,
                y: parts[1].parse()?,
                z: parts[2].parse()?,
            })
        } else {
            Ok(Atom::Name(s.into()))
        }
    }
}

impl Serialize for Atom {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'a> Deserialize<'a> for Atom {
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(AtomVisitor)
    }
}


/// Serde visitor for deserializing an Atom.
#[derive(Debug, Clone, Copy, Default)]
struct AtomVisitor;

impl<'a> Visitor<'a> for AtomVisitor {
    type Value = Atom;

    fn expecting(&self, f: &mut Formatter) -> FmtResult {
        f.write_str("an atom name or comma-separated X, Y, Z coordinates")
    }

    fn visit_str<E: DeError>(self, s: &str) -> Result<Self::Value, E> {
        Atom::from_str(s).map_err(E::custom)
    }
}

/// De/Serialize an invalid angle of -999.9 as `None`.
mod serde_angle {
    use serde::{
        ser::Serializer,
        de::{ Deserializer, Visitor, Error },
    };
    use std::fmt::{ Formatter, Result as FmtResult };

    /// Serialize a `None` angle as the invalid value -999.9.
    pub fn serialize<S: Serializer>(value: &Option<f64>, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_f64(value.unwrap_or(-999.9))
    }

    /// Deserialize an invalid angle of -999.9 as `None`.
    pub fn deserialize<'a, D: Deserializer<'a>>(d: D) -> Result<Option<f64>, D::Error> {
        d.deserialize_f64(AngleVisitor)
    }

    /// Visitor for `deserialize()`.
    #[derive(Debug, Clone, Copy, Default)]
    struct AngleVisitor;

    impl<'a> Visitor<'a> for AngleVisitor {
        type Value = Option<f64>;

        fn expecting(&self, f: &mut Formatter) -> FmtResult {
            f.write_str("a number representing an angle")
        }

        #[allow(clippy::float_cmp)]
        fn visit_f64<E: Error>(self, v: f64) -> Result<Self::Value, E> {
            if v == -999.9 {
                Ok(None)
            } else {
                Ok(Some(v))
            }
        }

        #[allow(clippy::cast_precision_loss)]
        fn visit_i64<E: Error>(self, v: i64) -> Result<Self::Value, E> {
            Ok(Some(v as f64))
        }

        #[allow(clippy::cast_precision_loss)]
        fn visit_u64<E: Error>(self, v: u64) -> Result<Self::Value, E> {
            Ok(Some(v as f64))
        }
    }
}

/// Deserialize an empty `NodeId` string as `None` instead of failing.
fn deserialize_empty_nodeid<'a, D: Deserializer<'a>>(d: D) -> Result<Option<NodeId>, D::Error> {
    d.deserialize_str(EmptyNodeIdVisitor)
}

/// Yields a `None` instead of an error when a NodeId is an empty string.
#[derive(Debug, Clone, Copy, Default)]
struct EmptyNodeIdVisitor;

impl<'a> Visitor<'a> for EmptyNodeIdVisitor {
    type Value = Option<NodeId>;

    fn expecting(&self, f: &mut Formatter) -> FmtResult {
        f.write_str("a NodeId or an empty string")
    }

    fn visit_str<E: DeError>(self, s: &str) -> Result<Self::Value, E> {
        if s.is_empty() {
            Ok(None)
        } else {
            NodeId::from_str(s).map(Some).map_err(E::custom)
        }
    }
}
