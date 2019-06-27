//! This binary crate demonstrates the use of the RING API client.

use std::env;
use std::thread;
use std::fmt::Debug;
use std::time::Duration;
use ring_api::{
    client::Client,
    requests::*,
    job::*,
    settings::*,
};

fn main() -> Result<(), Box<dyn Debug + 'static>> {
    // Parse command-line arguments
    let mut args: Vec<_> = env::args().collect();
    let err_msg = "usage: simple [id | file] PDB_ID_OR_FILE_NAME";
    let client = Client::new();

    if args.len() != 3 {
        return Err(Box::new(err_msg));
    }

    // Submit either a PDB ID or a PDB file
    let submit_resp = match args[1].as_str() {
        "id" => {
            let req = SubmitId {
                pdb_id: args.remove(2),
                settings: Settings::default(),
            };
            client.send(req).map_err(|e| Box::new(e) as _)?
        }
        "file" => {
            let req = SubmitStructure::with_pdb_file(&args[2])
                .map_err(|e| Box::new(e) as _)?;
            client.send(req).map_err(|e| Box::new(e) as _)?
        }
        _ => return Err(Box::new(err_msg)),
    };

    println!("Submitted; response = {:#?}", submit_resp);

    // Wait for the result to be ready; poll status endpoint
    loop {
        let req = Status {
            job_id: submit_resp.job_id.clone()
        };
        let status_resp = client.send(req).map_err(|e| Box::new(e) as _)?;

        println!("Job ID: {}, status: {}", status_resp.job_id, status_resp.status);

        match status_resp.status {
            JobStatus::Failed => return Err(Box::new(
                format!("Job failed: {:#?}", status_resp)
            )),
            JobStatus::Complete => break,
            _ => thread::sleep(Duration::from_secs(5)),
        }
    }

    // Retrieve results
    let req = RetrieveResult {
        job_id: submit_resp.job_id.clone()
    };
    let result_resp = client.send(req).map_err(|e| Box::new(e) as _)?;

    println!("{:#?}", result_resp);

    Ok(())
}
