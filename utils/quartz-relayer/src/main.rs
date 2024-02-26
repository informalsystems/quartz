mod cli;

use std::{
    fs::{read_to_string, File},
    io::Write,
    process::Command,
};

use clap::Parser;
use quartz_proto::quartz::{core_client::CoreClient, InstantiateRequest};
use quartz_relayer::types::InstantiateResponse;
use serde_json::json;

use crate::cli::Cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let mut client = CoreClient::connect(args.enclave_addr).await?;
    let response = client.instantiate(InstantiateRequest {}).await?;
    let response: InstantiateResponse = response.into_inner().try_into()?;

    let dir = tempfile::tempdir()?;
    let quote_file_path = dir.path().join("test.quote");
    let datareport_file_path = dir.path().join("datareport");
    let datareportsig_file_path = dir.path().join("datareportsig");

    let mut quote_file = File::create(quote_file_path)?;
    quote_file.write_all(response.quote())?;

    let gramine_sgx_ias_request_output = Command::new("gramine-sgx-ias-request")
        .arg("report")
        .args(["-g", "51CAF5A48B450D624AEFE3286D314894"])
        .args(["-k", "669244b3e6364b5888289a11d2a1726d"])
        .args(["-q", quote_file_path])
        .args(["-r", datareport_file_path])
        .args(["-s", datareportsig_file_path])
        .output()?;
    println!("{gramine_sgx_ias_request_output:?}");

    let report = read_to_string(datareport_file_path)?;
    let report_sig = read_to_string(datareportsig_file_path)?;
    let ias_report = json!({"report": report, "reportsig": report_sig});
    println!(
        "{}",
        serde_json::to_string(&ias_report).expect("infallible serializer")
    );

    Ok(())
}
