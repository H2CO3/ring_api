//! Parameters for submitting a job.

use std::str::FromStr;
use std::fmt::{ Display, Formatter, Result as FmtResult };
use serde::{
    ser::{ Serialize, Serializer, SerializeMap, Error },
    de::{ Deserialize, Deserializer, Visitor, MapAccess },
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
#[serde(default)]
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
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let thresholds_str = serde_json::to_string(&self.thresholds)
            .map_err(S::Error::custom)?;

        let mut map = serializer.serialize_map(Some(10))?;

        map.serialize_entry("ringmd", "false")?; // we always need this
        map.serialize_entry("chain", &self.chain)?;
        map.serialize_entry("networkPolicy", &self.network_policy)?;
        // !!! must be serialized as a string
        map.serialize_entry("seqSeparation", &self.sequence_separation.to_string())?;
        map.serialize_entry("thresholds", &thresholds_str)?;
        map.serialize_entry("nohetero", &self.skip_hetero.to_string())?;
        map.serialize_entry("nowater", &self.skip_water.to_string())?;
        map.serialize_entry("noenergy", &self.skip_energy.to_string())?;

        if self.perform_msa {
            map.serialize_entry("msa", "true")?;
        }

        match self.interactions {
            InteractionType::All => map.serialize_entry("allEdges", "true")?,
            InteractionType::Multiple => {},
            InteractionType::MostEnergetic => map.serialize_entry("onlyFirstEdge", "true")?,
            InteractionType::NoSpecific => map.serialize_entry("nospecific", "true")?,
        }

        map.end()
    }
}

impl<'de> Deserialize<'de> for Settings {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;

        /// Just because we are forced to use a visitor.
        #[derive(Debug, Clone, Copy)]
        struct SettingsVisitor;

        impl<'de> Visitor<'de> for SettingsVisitor {
            type Value = Settings;

            fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
                formatter.pad("a map of key-value pairs for settings")
            }

            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
                let mut settings = Settings::default();

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "chain" => {
                            settings.chain = map.next_value()?;
                        }
                        "networkPolicy" => {
                            settings.network_policy = map.next_value()?;
                        }
                        "allEdges" => {
                            let _: String = map.next_value()?;
                            settings.interactions = InteractionType::All;
                        }
                        "onlyFirstEdge" => {
                            let _: String = map.next_value()?;
                            settings.interactions = InteractionType::MostEnergetic;
                        }
                        "nospecific" => {
                            let _: String = map.next_value()?;
                            settings.interactions = InteractionType::NoSpecific;
                        }
                        "seqSeparation" => {
                            settings.sequence_separation = parse_next_value(&mut map)?;
                        }
                        "thresholds" => {
                            let value_str: String = map.next_value()?;
                            settings.thresholds = serde_json::from_str(
                                &value_str
                            ).map_err(
                                M::Error::custom
                            )?;
                        }
                        "nohetero" => {
                            settings.skip_hetero = parse_next_value(&mut map)?;
                        }
                        "nowater" => {
                            settings.skip_water = parse_next_value(&mut map)?;
                        }
                        "noenergy" => {
                            settings.skip_energy = parse_next_value(&mut map)?;
                        }
                        "msa" => {
                            let _: String = map.next_value()?;
                            settings.perform_msa = true;
                        }
                        _ => {
                            let _: String = map.next_value()?;
                        }
                    }
                }

                Ok(settings)
            }
        }

        deserializer.deserialize_map(SettingsVisitor)
    }
}

impl Serialize for Chain {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
            Chain::All => serializer.serialize_str("all"),
            Chain::Id(id) => serializer.serialize_char(id),
        }
    }
}

impl<'de> Deserialize<'de> for Chain {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;

        /// Just because we are forced to use a visitor.
        #[derive(Debug, Clone, Copy)]
        struct ChainVisitor;

        impl<'de> Visitor<'de> for ChainVisitor {
            type Value = Chain;

            fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
                formatter.pad("a one-letter chain ID or \"all\"")
            }

            fn visit_char<E: Error>(self, v: char) -> Result<Self::Value, E> {
                Ok(Chain::Id(v))
            }

            fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
                if v == "all" {
                    Ok(Chain::All)
                } else {
                    let mut chars = v.chars();

                    if let Some(c) = chars.next() {
                        if chars.next().is_none() {
                            Ok(Chain::Id(c))
                        } else {
                            Err(E::custom("multi-letter string is not a valid chain ID"))
                        }
                    } else {
                       Err(E::custom("empty string is not a valid chain ID"))
                    }
                }
            }
        }

        deserializer.deserialize_str(ChainVisitor)
    }
}

/// Private helper for deserializing a map.
fn parse_next_value<'de, T, M>(map: &mut M) -> Result<T, M::Error>
    where T: FromStr,
          T::Err: Display,
          M: MapAccess<'de>,
{
    use serde::de::Error;
    let value_str: String = map.next_value()?;
    value_str.parse().map_err(M::Error::custom)
}
