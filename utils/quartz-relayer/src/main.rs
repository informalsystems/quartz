mod cli;

use std::{fs::File, io::Write};

use clap::Parser;
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

    Ok(())
}
