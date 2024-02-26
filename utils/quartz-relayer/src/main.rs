mod cli;

use std::{
    error::Error,
    fs::{read_to_string, File},
    io::Write,
    process::Command,
};

use clap::Parser;
use quartz_proto::quartz::{core_client::CoreClient, InstantiateRequest};
use quartz_relayer::types::InstantiateResponse;
use serde_json::{json, Value};

use crate::cli::Cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let mut client = CoreClient::connect(args.enclave_addr).await?;
    let response = client.instantiate(InstantiateRequest {}).await?;
    let response: InstantiateResponse = response.into_inner().try_into()?;

    let ias_report = gramine_sgx_ias_report(response.quote())?;
    println!(
        "{}",
        serde_json::to_string(&ias_report).expect("infallible serializer")
    );

    Ok(())
}

fn gramine_sgx_ias_report(quote: &[u8]) -> Result<Value, Box<dyn Error>> {
    let dir = tempfile::tempdir()?;
    let quote_file_path = dir.path().join("test.quote");
    let datareport_file_path = dir.path().join("datareport");
    let datareportsig_file_path = dir.path().join("datareportsig");

    let mut quote_file = File::create(quote_file_path.clone())?;
    quote_file.write_all(quote)?;

    let gramine_sgx_ias_request_output = Command::new("gramine-sgx-ias-request")
        .arg("report")
        .args(["-g", "51CAF5A48B450D624AEFE3286D314894"])
        .args(["-k", "669244b3e6364b5888289a11d2a1726d"])
        .args(["-q", &quote_file_path.display().to_string()])
        .args(["-r", &datareport_file_path.display().to_string()])
        .args(["-s", &datareportsig_file_path.display().to_string()])
        .output()?;
    println!("{gramine_sgx_ias_request_output:?}");

    let report = read_to_string(datareport_file_path)?;
    let report_sig = read_to_string(datareportsig_file_path)?;
    let ias_report = json!({"report": report, "reportsig": report_sig});
    Ok(ias_report)
}
