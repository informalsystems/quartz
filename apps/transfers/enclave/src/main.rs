// #![doc = include_str!("../README.md")]
// #![forbid(unsafe_code)]
// #![warn(
//     clippy::checked_conversions,
//     clippy::panic,
//     clippy::panic_in_result_fn,
//     trivial_casts,
//     trivial_numeric_casts,
//     rust_2018_idioms,
//     unused_lifetimes,
//     unused_import_braces,
//     unused_qualifications
// )]

// pub mod cli;
// pub mod proto;
// pub mod state;
// pub mod transfers_server;
// pub mod wslistener;

// use std::{
//     sync::{Arc, Mutex},
//     time::Duration,
// };

// use clap::Parser;
// use cli::Cli;
// use quartz_common::{
//     contract::state::{Config, LightClientOpts},
//     enclave::{
//         attestor::{Attestor, DcapAttestor},
//         server::{QuartzServer, WsListenerConfig},
//     },
// };
// use transfers_server::{TransfersOp, TransfersService};
// use tokio::sync::mpsc;
// use crate::wslistener::WsListener;

// #[tokio::main(flavor = "current_thread")]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let args = Cli::parse();

//     let light_client_opts = LightClientOpts::new(
//         args.chain_id.clone(),
//         args.trusted_height.into(),
//         Vec::from(args.trusted_hash)
//             .try_into()
//             .expect("invalid trusted hash"),
//         (
//             args.trust_threshold.numerator(),
//             args.trust_threshold.denominator(),
//         ),
//         args.trusting_period,
//         args.max_clock_drift,
//         args.max_block_lag,
//     )?;

//     #[cfg(not(feature = "mock-sgx"))]
//     let attestor = DcapAttestor {
//         fmspc: args.fmspc.expect("FMSPC is required for DCAP"),
//     };

//     #[cfg(feature = "mock-sgx")]
//     let attestor = attestor::MockAttestor::default();

//     let config = Config::new(
//         attestor.mr_enclave()?,
//         Duration::from_secs(30 * 24 * 60),
//         light_client_opts,
//         args.tcbinfo_contract.map(|c| c.to_string()),
//         args.dcap_verifier_contract.map(|c| c.to_string()),
//     );

//     let ws_config = WsListenerConfig {
//         node_url: args.node_url,
//         websocket_url: args.websocket_url,
//         tx_sender: args.tx_sender,
//         trusted_hash: args.trusted_hash,
//         trusted_height: args.trusted_height,
//         chain_id: args.chain_id,
//     };


//     let sk = Arc::new(Mutex::new(None));

//     // Event queue
//     let (tx, mut rx) = mpsc::channel::<TransfersOp<DcapAttestor>>(1);
//     // Consumer task: dequeue and process events
//     tokio::spawn(async move {
//         while let Some(op) = rx.recv().await {
//             if let Err(e) = op.client.process(op.event, op.config).await {
//                 println!("Error processing queued event: {}", e);
//             }
//         }
//     });

//     QuartzServer::new(config.clone(), sk.clone(), attestor.clone(), ws_config)
//         .add_service(TransfersService::new(config, sk, attestor, tx))
//         .serve(args.rpc_addr)
//         .await?;

//     Ok(())
// }
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

pub mod cli;
pub mod proto;
pub mod state;
pub mod transfers_server;
pub mod wslistener;

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use clap::Parser;
use cli::Cli;
use quartz_common::{
    contract::state::{Config, LightClientOpts},
    enclave::{
        attestor::{Attestor, DcapAttestor},
        server::{QuartzServer, WsListenerConfig},
    },
};
use transfers_server::TransfersService;
use wslistener::start_ws_listener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let args = Cli::parse();

    let light_client_opts = LightClientOpts::new(
        args.chain_id.clone(),
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
    let attestor = DcapAttestor {
        fmspc: args.fmspc.expect("FMSPC is required for DCAP"),
    };

    #[cfg(feature = "mock-sgx")]
    let attestor = attestor::MockAttestor::default();

    let config = Config::new(
        attestor.mr_enclave()?,
        Duration::from_secs(30 * 24 * 60),
        light_client_opts,
        args.tcbinfo_contract.map(|c| c.to_string()),
        args.dcap_verifier_contract.map(|c| c.to_string()),
    );

    let ws_config = WsListenerConfig {
        node_url: args.node_url,
        websocket_url: args.websocket_url,
        tx_sender: args.tx_sender,
        trusted_hash: args.trusted_hash,
        trusted_height: args.trusted_height,
        chain_id: args.chain_id,
    };

    let sk = Arc::new(Mutex::new(None));

    let transfers_service = TransfersService::new(config.clone(), sk.clone(), attestor.clone());

    // Start the WebSocket listener in a separate task
    let ws_listener_handle = tokio::spawn(start_ws_listener(transfers_service.clone(), ws_config.clone()));

    // Start the QuartzServer
    let quartz_server_handle = tokio::spawn(async move {
        QuartzServer::new(config, sk, attestor, ws_config)
            .add_service(transfers_service)
            .serve(args.rpc_addr)
            .await
    });

    // Wait for both the WebSocket listener and QuartzServer to complete
    tokio::try_join!(
        async { ws_listener_handle.await.expect("WebSocket listener task failed") },
        async { quartz_server_handle.await.expect("QuartzServer task failed")? }
    )?;

    Ok(())
}