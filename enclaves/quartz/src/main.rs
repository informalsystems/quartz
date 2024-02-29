#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(
    clippy::checked_conversions,
    clippy::panic,
    clippy::panic_in_result_fn,
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
mod proto;
mod server;

use std::time::Duration;

use clap::Parser;
use quartz_cw::state::{Config, LightClientOpts};
use quartz_proto::quartz::core_server::CoreServer;
use tonic::transport::Server;

use crate::{
    attestor::{Attestor, EpidAttestor},
    cli::Cli,
    server::CoreService,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let light_client_opts = LightClientOpts::new(
        args.chain_id,
        args.trusted_height.into(),
        Vec::from(args.trusted_hash)
            .try_into()
            .expect("invalid trusted hash"),
        (
            args.trust_threshold.numerator(),
            args.trust_threshold.denominator(),
        ),
        args.trusting_period,
        args.max_clock_drift,
        args.max_block_lag,
    )?;

    let config = Config::new(
        EpidAttestor.mr_enclave()?,
        Duration::from_secs(30 * 24 * 60),
        light_client_opts,
    );

    Server::builder()
        .add_service(CoreServer::new(CoreService::new(config, EpidAttestor)))
        .serve(args.rpc_addr)
        .await?;

    Ok(())
}
