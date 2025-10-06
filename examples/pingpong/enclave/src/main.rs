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
pub mod event;
pub mod proto;
pub mod request;

use clap::Parser;
use cli::Cli;
use cosmrs::AccountId;
use quartz_common::{
    contract::state::{Config, LightClientOpts},
    enclave::{
        attestor::{self, Attestor},
        chain_client::{
            default::{DefaultChainClient, DefaultTxConfig},
            ChainClient,
        },
        host::{DefaultHost, GasProvider, Host},
        DefaultSharedEnclave,
    },
};

use crate::{
    event::EnclaveEvent,
    request::{EnclaveRequest, EnclaveResponse},
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .write_style(env_logger::WriteStyle::Always)
        .init();

    let args = Cli::parse();

    let sk = {
        let sk = std::env::var("ADMIN_SK")
            .map_err(|_| anyhow::anyhow!("Admin secret key not found in env vars"))?;
        hex::decode(sk)?
            .as_slice()
            .try_into()
            .map_err(|e| anyhow::anyhow!("failed to read/parse sk: {}", e))?
    };

    let light_client_opts = LightClientOpts::new(
        args.chain_id.to_string(),
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
    let attestor = attestor::DcapAttestor {
        fmspc: args.fmspc.expect("FMSPC is required for DCAP"),
        pccs_url: args.pccs_url.expect("PCCS URL is required for DCAP"),
    };

    #[cfg(feature = "mock-sgx")]
    let attestor = attestor::MockAttestor::default();

    let config = Config::new(
        attestor.mr_enclave()?,
        light_client_opts,
        args.tcbinfo_contract.map(|c| c.to_string()),
        args.dcap_verifier_contract.map(|c| c.to_string()),
    );
    let chain_client = DefaultChainClient::new(
        args.chain_id,
        sk,
        args.grpc_url,
        args.node_url,
        args.ws_url.clone(),
        args.trusted_height,
        args.trusted_hash,
    );

    let (enclave, notifier_rx) = DefaultSharedEnclave::shared(attestor, config, ());

    let host = DefaultHost::<EnclaveRequest, EnclaveEvent, _, _>::new(
        enclave,
        chain_client,
        GasSimulator,
        args.backup_path,
        notifier_rx,
    );

    host.serve(args.ws_url, args.rpc_addr).await?;

    Ok(())
}

struct GasSimulator;

#[async_trait::async_trait]
impl GasProvider<EnclaveResponse, DefaultChainClient> for GasSimulator {
    async fn gas_for_tx(
        &self,
        tx: &EnclaveResponse,
        chain_client: &DefaultChainClient,
        contract: &AccountId,
    ) -> Result<DefaultTxConfig, anyhow::Error> {
        let default_config = DefaultTxConfig {
            gas: 2000000,
            amount: "11000untrn".to_string(),
        };
        let gas_info = chain_client
            .simulate_tx(contract, tx.as_slice().iter(), default_config)
            .await?;
        Ok(DefaultTxConfig::new(
            gas_info.gas_used,
            1.3,
            0.0053,
            "untrn",
        ))
    }
}
