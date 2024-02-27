#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(
    clippy::checked_conversions,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_used,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    rust_2018_idioms,
    unused_lifetimes,
    unused_import_braces,
    unused_qualifications
)]

mod attestor;
mod cli;
mod server;

use std::{process::Command, time::Duration};

use clap::Parser;
use cosmwasm_std::HexBinary;
use quartz_cw::state::{Config, LightClientOpts};
use quartz_proto::quartz::core_server::CoreServer;
use tonic::transport::Server;

use crate::{attestor::EpidAttestor, cli::Cli, server::CoreService};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let gramine_sgx_sigstruct_view = Command::new("gramine-sgx-sigstruct-view")
        .args(["--output-format", "json"])
        .arg(args.sigfile)
        .output()?;

    let sigstruct_json: serde_json::Value =
        serde_json::from_str(&String::from_utf8(gramine_sgx_sigstruct_view.stdout)?)?;
    let mr_enclave = HexBinary::from_hex(&sigstruct_json["mr_enclave"].to_string())?.to_array()?;
    let light_client_opts = LightClientOpts::new(
        args.chain_id,
        args.trusted_height,
        args.trusted_hash,
        args.trust_threshold,
        args.trusting_period,
        args.max_clock_drift,
        args.max_block_lag,
    );
    let config = Config::new(
        mr_enclave,
        Duration::from_secs(30 * 24 * 60),
        light_client_opts,
    );

    Server::builder()
        .add_service(CoreServer::new(CoreService::new(config, EpidAttestor)))
        .serve(args.rpc_addr)
        .await?;

    Ok(())
}
