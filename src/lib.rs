//! This library provides a convenient interface to the RING webservice,
//! a software for computing protein interaction networks.

#![doc(html_root_url = "https://docs.rs/ring_api/0.1.0")]
#![deny(missing_debug_implementations, missing_copy_implementations,
        trivial_casts, trivial_numeric_casts,
        unsafe_code,
        unstable_features,
        anonymous_parameters, bare_trait_objects,
        variant_size_differences,
        unused_import_braces, unused_qualifications, missing_docs)]
#![allow(clippy::single_match, clippy::match_same_arms, clippy::match_ref_pats,
         clippy::clone_on_ref_ptr, clippy::needless_pass_by_value)]
#![deny(clippy::wrong_pub_self_convention, clippy::used_underscore_binding,
        clippy::similar_names, clippy::pub_enum_variant_names,
        clippy::missing_docs_in_private_items,
        clippy::non_ascii_literal, clippy::unicode_not_nfc,
        clippy::result_unwrap_used, clippy::option_unwrap_used,
        clippy::option_map_unwrap_or_else, clippy::option_map_unwrap_or,
        clippy::filter_map,
        clippy::shadow_unrelated, clippy::shadow_reuse, clippy::shadow_same,
        clippy::int_plus_one, clippy::string_add_assign, clippy::if_not_else,
        clippy::invalid_upcast_comparisons,
        clippy::cast_precision_loss, clippy::cast_lossless,
        clippy::cast_possible_wrap, clippy::cast_possible_truncation,
        clippy::mutex_integer, clippy::mut_mut, clippy::items_after_statements,
        clippy::print_stdout, clippy::mem_forget, clippy::maybe_infinite_iter)]

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate reqwest;

pub use client::*;
pub use error::*;
pub use requests::*;
pub use settings::*;
pub use job::*;

pub mod client;
pub mod error;
pub mod requests;
pub mod settings;
pub mod job;
pub mod multipart;

#[cfg(test)]
#[allow(clippy::print_stdout)]
mod tests {
    use super::*;

    #[test]
    fn submit_id() -> Result<()> {
        use std::{
            thread::sleep,
            time::Duration,
        };

        let client = Client::new();
        let request = SubmitId {
            pdb_id: String::from("3S6A"),
            settings: Settings {
                chain: Chain::Id('A'),
                network_policy: NetworkPolicy::CAlpha,
                interactions: InteractionType::All,
                ..Default::default()
            },
        };
        let response = client.send(&request)?;
        println!("{:#?}", response);

        loop {
            let request = Status {
                job_id: response.job_id.clone(),
            };
            let response = client.send(&request)?;

            println!("{:#?}", response);

            match response.status {
                JobStatus::Complete => break Ok(()),
                JobStatus::Failed => panic!("job failed"),
                JobStatus::InProgress | JobStatus::Partial => {}
            }

            sleep(Duration::from_secs(5));
        }
    }

    #[test]
    pub fn submit_structure() -> Result<()> {
        use std::{
            thread::sleep,
            time::Duration,
        };

        let client = Client::new();
        let request = SubmitStructure::with_pdb_file("testdata/3s6a.pdb")?;
        let response = client.send(&request)?;
        println!("{:#?}", response);

        loop {
            let request = Status {
                job_id: response.job_id.clone(),
            };
            let response = client.send(&request)?;

            println!("{:#?}", response);

            match response.status {
                JobStatus::Complete => break Ok(()),
                JobStatus::Failed => panic!("job failed"),
                JobStatus::InProgress | JobStatus::Partial => {}
            }

            sleep(Duration::from_secs(5));
        }
    }

    #[test]
    fn serde() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let s1 = Settings {
            chain: Chain::Id('X'),
            network_policy: NetworkPolicy::CBeta,
            interactions: InteractionType::NoSpecific,
            sequence_separation: 42,
            thresholds: Thresholds::relaxed(),
            skip_hetero: true,
            skip_water: false,
            perform_msa: true,
            ..Default::default()
        };
        let json = serde_json::to_string_pretty(&s1)?;
        let s2: Settings = serde_json::from_str(&json)?;
        assert_eq!(s1, s2);

        let json_default = serde_json::to_string(&Settings::default())?;
        let s_default: Settings = serde_json::from_str(&json_default)?;
        assert_eq!(s_default, Settings::default());

        Ok(())
    }

    #[test]
    fn retrieve_result() -> Result<()> {
        let client = Client::new();
        let job_id = JobId::from("5cefd030b265bd294b0f6b2c");
        let request = RetrieveResult { job_id };
        let response = client.send(&request)?;

        println!("{:#?}", response.nodes[0]);

        Ok(())
    }
}
