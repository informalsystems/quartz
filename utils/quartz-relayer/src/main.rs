mod cli;

use std::{fs::File, io::Write};
use std::fs::read_to_string;
use std::process::Command;

use clap::Parser;
use serde_json::json;
use quartz_proto::quartz::{core_client::CoreClient, InstantiateRequest};
use quartz_relayer::types::InstantiateResponse;

use crate::{cli::Cli};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let mut client = CoreClient::connect(args.enclave_addr).await?;
    let response = client.instantiate(InstantiateRequest {}).await?;
    let response: InstantiateResponse = response.into_inner().try_into()?;

    let mut quote_file = File::create("test.quote")?;
    quote_file.write_all(response.quote())?;

    let gramine_sgx_ias_request_output = Command::new("gramine-sgx-ias-request")
        .arg("report")
        .args(["-g", "51CAF5A48B450D624AEFE3286D314894"])
        .args(["-k", "669244b3e6364b5888289a11d2a1726d"])
        .args(["-q", "test.quote"])
        .args(["-r", "datareport"])
        .args(["-s", "datareportsig"])
        .output()?;
    println!("{gramine_sgx_ias_request_output:?}");

    let report = read_to_string("datareport")?;
    let report_sig = read_to_string("datareportsig")?;
    let ias_report = json!({"report": report, "reportsig": report_sig});
    println!("{}", serde_json::to_string(&ias_report).expect("infallible serializer"));

    Ok(())
}
