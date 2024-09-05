#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(
    clippy::checked_conversions,
    clippy::panic,
    clippy::panic_in_result_fn,
    trivial_casts,
    trivial_numeric_casts,
    rust_2018_idioms,
    unused_lifetimes,
    unused_import_braces,
    unused_qualifications
)]

mod cli;
mod mtcs_server;
mod proto;
mod types;

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use clap::Parser;
use cli::Cli;
use mtcs_server::MtcsService;
use proto::clearing_server::ClearingServer as MtcsServer;
use quartz_common::{
    contract::state::{Config, LightClientOpts},
    enclave::{
        attestor::{self, Attestor},
        server::CoreService,
    },
    proto::core_server::CoreServer,
};
use tonic::transport::Server;

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

    #[cfg(not(feature = "mock-sgx"))]
    let attestor = attestor::DcapAttestor { fmspc: args.fmspc };

    #[cfg(feature = "mock-sgx")]
    let attestor = attestor::MockAttestor;

    let config: Config = Config::new(
        attestor.mr_enclave()?,
        Duration::from_secs(30 * 24 * 60),
        light_client_opts,
        args.tcbinfo_contract.to_string(),
    );

    let sk = Arc::new(Mutex::new(None));

    Server::builder()
        .add_service(CoreServer::new(CoreService::new(
            config.clone(),
            sk.clone(),
            attestor.clone(),
        )))
        .add_service(MtcsServer::new(MtcsService::new(config, sk, attestor)))
        .serve(args.rpc_addr)
        .await?;

    Ok(())
}
