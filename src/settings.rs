//! Parameters for submitting a job.

use serde::{
    ser::{ Serialize, Serializer },
    de::{ Deserialize, Deserializer },
};

/// Parameters for submitting a job.
/// See the [documentation](http://protein.bio.unipd.it/ring/help#params)
/// of each parameter for a detailed explanation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Settings {
    /// Which chain(s) to consider when computing interactions. Default `All`.
    pub chain: Chain,
    /// Which set of atoms to consider when evaluating contacts. Default `Closest`.
    pub network_policy: NetworkPolicy,
    /// How many interactions to report per edge per type. Default `Multiple`.
    pub interactions: InteractionType,
    /// Max distances between two pairs of atoms to consider as interacting. Default `strict`.
    pub thresholds: Thresholds,
    /// Maximal distance between residues to consider at once. Default 3.
    pub sequence_separation: usize,
    /// Skip hetero atoms and ligands. Default `false`.
    pub skip_hetero: bool,
    /// Skip water (solvent) molecules. Default `true`.
    pub skip_water: bool,
    /// Skip FRST and TAP energy calculation. Default `true`.
    pub skip_energy: bool,
    /// Calculate mutual information from a multiple alignment via BLAST (slow!)
    /// Default `false`.
    pub perform_msa: bool,
}

/// Chain ID for computing a single chain or "all" for computing all chains.
/// The default is `All`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Chain {
    /// Compute all chains.
    All,
    /// Compute a single chain with the given character ID.
    Id(char),
}

/// Which atoms to consider when computing interactions.
/// The default is `Closest`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NetworkPolicy {
    /// Measure distance between all atoms of the two residues.
    #[serde(rename = "closest")]
    Closest,
    /// Measure distance between the two centers of mass.
    #[serde(rename = "lollipop")]
    Lollipop,
    /// Measure distance between the two alpha carbon atoms.
    #[serde(rename = "ca")]
    CAlpha,
    /// Measure distance between the two beta carbon atoms.
    #[serde(rename = "cb")]
    CBeta,
}

/// Which interaction(s) to return for each edge (pair of interacting residues).
/// The default is `Multiple`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionType {
    /// Return all interactions found.
    All,
    /// Return multiple types of interactions, but only one per type.
    Multiple,
    /// Only return one interaction: the most energetic one, regardless of type.
    MostEnergetic,
    /// Only return generic IAC interactions (whatever that means).
    NoSpecific,
}

/// Distance thresholds (maximum) between atoms.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Thresholds {
    /// Hydrogen bonds.
    #[serde(rename = "hbond")]
    pub hydrogen: f32,
    /// Van der Waals-forces
    #[serde(rename = "vdw")]
    pub van_der_waals: f32,
    /// Ionic interactions, salt bridges
    #[serde(rename = "ionic")]
    pub ionic: f32,
    /// Pi-Pi stacking
    #[serde(rename = "pipi")]
    pub pi_pi: f32,
    /// Pi-cation interactions
    #[serde(rename = "pication")]
    pub pi_cation: f32,
    /// Disulphide bonds
    #[serde(rename = "disulphide")]
    pub disulphide: f32,
}

impl Thresholds {
    /// Thresholds suitable for generating a reliable network.
    pub fn strict() -> Self {
        Thresholds {
            hydrogen: 3.5,
            van_der_waals: 0.5,
            ionic: 4.0,
            pi_pi: 6.5,
            pi_cation: 5.0,
            disulphide: 2.5,
        }
    }

    /// Thresholds suitable for generating an inclusive network.
    pub fn relaxed() -> Self {
        Thresholds {
            hydrogen: 5.5,
            van_der_waals: 0.8,
            ionic: 5.0,
            pi_pi: 7.0,
            pi_cation: 7.0,
            disulphide: 3.0,
        }
    }
}

// Default impls

impl Default for Settings {
    fn default() -> Self {
        Settings {
            chain: Chain::default(),
            network_policy: NetworkPolicy::default(),
            interactions: InteractionType::default(),
            thresholds: Thresholds::default(),
            sequence_separation: 3,
            skip_hetero: false,
            skip_water: true,
            skip_energy: true,
            perform_msa: false,
        }
    }
}

impl Default for Chain {
    fn default() -> Self {
        Chain::All
    }
}

impl Default for NetworkPolicy {
    fn default() -> Self {
        NetworkPolicy::Closest
    }
}

impl Default for InteractionType {
    fn default() -> Self {
        InteractionType::Multiple
    }
}

impl Default for Thresholds {
    /// The default is `strict`.
    fn default() -> Self {
        Thresholds::strict()
    }
}

// Serialize and Deserialize impls

impl Serialize for Settings {
    fn serialize<S: Serializer>(&self, mut serializer: S) -> Result<S::Ok, S::Error> {
        unimplemented!()
    }
}

impl<'de> Deserialize<'de> for Settings {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        unimplemented!()
    }
}
