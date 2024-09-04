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
mod wslistener;

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
        attestor::{Attestor, DefaultAttestor},
        server::{QuartzServer, WsListenerConfig},
    },
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

    let attestor = DefaultAttestor::default();

    let config: Config = Config::new(
        attestor.mr_enclave()?,
        Duration::from_secs(30 * 24 * 60),
        light_client_opts,
    );

    let ws_config = WsListenerConfig {
        node_url: "143.244.186.205:26657".to_string(),
        tx_sender: "admin".to_string(),
    };

    let sk = Arc::new(Mutex::new(None));

    QuartzServer::new(config.clone(), sk.clone(), attestor.clone(), ws_config)
        .add_service(MtcsServer::new(MtcsService::new(config, sk, attestor)))
        .serve(args.rpc_addr)
        .await?;

    Ok(())
}
