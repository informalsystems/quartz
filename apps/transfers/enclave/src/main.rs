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

use std::{
    sync::{Arc, Mutex}, time::Duration
};

use clap::Parser;
use cli::Cli;
use futures_util::StreamExt;
use proto::{settlement_server::SettlementServer as TransfersServer, event_listener_server::EventListenerServer};
use quartz_common::{
    contract::state::{Config, LightClientOpts},
    enclave::{
        attestor::{Attestor, DefaultAttestor},
        server::{CoreService, QuartzServer, WebSocketListener},
    },
    proto::core_server::CoreServer,
};
use tendermint_rpc::{query::{EventType, Query}, SubscriptionClient, WebSocketClient};
use transfers_server::TransfersService;

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

    let config = Config::new(
        attestor.mr_enclave()?,
        Duration::from_secs(30 * 24 * 60),
        light_client_opts,
    );

    let sk = Arc::new(Mutex::new(None));

    QuartzServer::new(config.clone(), sk.clone(), attestor.clone())
        .add_service(TransfersServer::new(TransfersService::new(
            config, sk, attestor,
        )))
        .serve(args.rpc_addr)
        .await?;


    Ok(())
}

// TODO: Need to prevent listener from taking actions until handshake is completed
#[async_trait::async_trait]
impl<A: Attestor> WebSocketListener for TransfersServer<TransfersService<A>> {
    async fn listen(&self) -> Result<(), tonic::transport::Error> {
        let wsurl = format!("ws://143.244.186.205:26657/websocket");
        let (client, driver) = WebSocketClient::new(wsurl.as_str()).await.unwrap();
        let driver_handle = tokio::spawn(async move { driver.run().await });
    
        // let mut subs = client
        //     .subscribe(Query::from(EventType::Tx).and_contains("wasm.action", "init_clearing"))
        //     .await
        //     .unwrap();

        let mut subs = client
            .subscribe(Query::from(EventType::NewBlock))
            .await
            .unwrap();
    
        while subs.next().await.is_some() {
            // On init_clearing, run process
            println!("Saw a block!");
        }
    
        // Close connection
        // Await the driver's termination to ensure proper connection closure.
        client.close().unwrap();
        let _ = driver_handle.await.unwrap();

        Ok(())
    }
}
    