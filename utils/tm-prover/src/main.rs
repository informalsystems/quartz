#![deny(
    warnings,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications
)]
#![forbid(unsafe_code)]

use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
    str::FromStr,
    time::Duration,
};

use clap::Parser;
use color_eyre::{
    eyre::{eyre, Result},
    Report,
};
use cosmrs::AccountId;
use cw_proof::{
    error::ProofError,
    proof::{
        cw::{CwProof, RawCwProof},
        key::CwAbciKey,
        Proof,
    },
};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use tendermint::{crypto::default::Sha256, evidence::Evidence};
use tendermint_light_client::{
    builder::LightClientBuilder,
    light_client::Options,
    store::memory::MemoryStore,
    types::{Hash, Height, LightBlock, TrustThreshold},
};
use tendermint_light_client_detector::{detect_divergence, Error, Provider, Trace};
use tendermint_rpc::{client::HttpClient, Client, HttpClientUrl};
use tracing::{error, info, metadata::LevelFilter};
use tracing_subscriber::{util::SubscriberInitExt, EnvFilter};

const WASM_STORE_KEY: &str = "/store/wasm/key";

fn parse_trust_threshold(s: &str) -> Result<TrustThreshold> {
    if let Some((l, r)) = s.split_once('/') {
        TrustThreshold::new(l.parse()?, r.parse()?).map_err(Into::into)
    } else {
        Err(eyre!(
            "invalid trust threshold: {s}, format must be X/Y where X and Y are integers"
        ))
    }
}

#[derive(Clone, Debug)]
struct List<T>(Vec<T>);

impl<E, T: FromStr<Err = E>> FromStr for List<T> {
    type Err = E;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split(',')
            .map(|s| s.parse())
            .collect::<Result<Vec<_>, _>>()
            .map(Self)
    }
}

#[derive(clap::Args, Debug, Clone)]
struct Verbosity {
    /// Increase verbosity, can be repeated up to 2 times
    #[arg(long, short, action = clap::ArgAction::Count)]
    verbose: u8,
}

impl Verbosity {
    fn to_level_filter(&self) -> LevelFilter {
        match self.verbose {
            0 => LevelFilter::INFO,
            1 => LevelFilter::DEBUG,
            _ => LevelFilter::TRACE,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ProofOutput {
    light_client_proof: Vec<LightBlock>,
    merkle_proof: RawCwProof,
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Identifier of the chain
    #[clap(long)]
    chain_id: String,

    /// Primary RPC address
    #[clap(long, default_value = "http://127.0.0.1:26657")]
    primary: HttpClientUrl,

    /// Comma-separated list of witnesses RPC addresses
    #[clap(long)]
    witnesses: List<HttpClientUrl>,

    /// Height of trusted header
    #[clap(long)]
    trusted_height: Height,

    /// Hash of trusted header
    #[clap(long)]
    trusted_hash: Hash,

    /// Trust threshold
    #[clap(long, value_parser = parse_trust_threshold, default_value_t = TrustThreshold::TWO_THIRDS)]
    trust_threshold: TrustThreshold,

    /// Trusting period, in seconds (default: two weeks)
    #[clap(long, default_value = "1209600")]
    trusting_period: u64,

    /// Maximum clock drift, in seconds
    #[clap(long, default_value = "5")]
    max_clock_drift: u64,

    /// Maximum block lag, in seconds
    #[clap(long, default_value = "5")]
    max_block_lag: u64,

    /// Output file to store light client proof (AKA verification trace)
    #[clap(long)]
    trace_file: Option<PathBuf>,

    /// Increase verbosity
    #[clap(flatten)]
    verbose: Verbosity,

    /// Address of the CosmWasm contract
    #[clap(long)]
    contract_address: AccountId,

    /// Storage key of the state item for which proofs must be retrieved
    #[clap(long)]
    storage_key: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Cli::parse();

    let env_filter = EnvFilter::builder()
        .with_default_directive(args.verbose.to_level_filter().into())
        .from_env_lossy();

    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(env_filter)
        .finish()
        .init();

    let options = Options {
        trust_threshold: args.trust_threshold,
        trusting_period: Duration::from_secs(args.trusting_period),
        clock_drift: Duration::from_secs(args.max_clock_drift),
    };

    let mut primary = make_provider(
        &args.chain_id,
        args.primary.clone(),
        args.trusted_height,
        args.trusted_hash,
        options,
    )
    .await?;

    let client = HttpClient::builder(args.primary.clone()).build()?;

    let trusted_block = primary
        .latest_trusted()
        .ok_or_else(|| eyre!("No trusted state found for primary"))?;

    let status = client.status().await?;
    let latest_height = status.sync_info.latest_block_height;

    // `proof_height` is the height at which we want to query the blockchain's state
    // This is one less than than the `latest_height` because we want to verify the merkle-proof for
    // the state against the `app_hash` at `latest_height`.
    // (because Tendermint commits to the latest `app_hash` in the subsequent block)
    let proof_height = (latest_height.value() - 1)
        .try_into()
        .expect("infallible conversion");

    info!("Verifying to latest height on primary...");

    let primary_block = primary.verify_to_height(latest_height)?;

    info!("Verified to height {} on primary", primary_block.height());
    let mut primary_trace = primary.get_trace(primary_block.height());

    let witnesses = join_all(args.witnesses.0.into_iter().map(|addr| {
        make_provider(
            &args.chain_id,
            addr,
            trusted_block.height(),
            trusted_block.signed_header.header.hash(),
            options,
        )
    }))
    .await;

    let mut witnesses = witnesses.into_iter().collect::<Result<Vec<_>>>()?;

    let max_clock_drift = Duration::from_secs(args.max_clock_drift);
    let max_block_lag = Duration::from_secs(args.max_block_lag);

    run_detector(
        &mut primary,
        witnesses.as_mut_slice(),
        primary_trace.clone(),
        max_clock_drift,
        max_block_lag,
    )
    .await?;

    // Up until now, we have been generating and verifying the consensus proof

    let status = client.status().await?;
    let latest_app_hash = primary_block.signed_header.header.app_hash;

    let path = WASM_STORE_KEY.to_owned();
    let data = CwAbciKey::new(args.contract_address, args.storage_key, None);
    let result = client
        .abci_query(Some(path), data, Some(proof_height), true)
        .await?;

    let proof: CwProof = result
        .clone()
        .try_into()
        .expect("result should contain proof");
    proof
        .verify(latest_app_hash.clone().into())
        .map_err(|e: ProofError| eyre!(e))?;

    if let Some(trace_file) = args.trace_file {
        // replace the last block in the trace (i.e. the (latest - 1) block) with the latest block
        // we don't actually verify the latest block because it will be verified on the other side
        let latest_block = primary.fetch_light_block(status.sync_info.latest_block_height)?;
        let _ = primary_trace.pop();
        primary_trace.push(latest_block);

        let output = ProofOutput {
            light_client_proof: primary_trace,
            merkle_proof: proof.into(),
        };
        write_proof_to_file(trace_file, output).await?;
    };

    Ok(())
}

async fn write_proof_to_file(trace_file: PathBuf, output: ProofOutput) -> Result<()> {
    info!("Writing proof to output file ({})", trace_file.display());

    let file = File::create(trace_file)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &output)?;
    writer.flush()?;
    Ok(())
}

async fn run_detector(
    primary: &mut Provider,
    witnesses: &mut [Provider],
    primary_trace: Vec<LightBlock>,
    max_clock_drift: Duration,
    max_block_lag: Duration,
) -> Result<(), Report> {
    if witnesses.is_empty() {
        return Err(Error::no_witnesses().into());
    }

    info!(
        "Running misbehavior detection against {} witnesses...",
        witnesses.len()
    );

    let primary_trace = Trace::new(primary_trace)?;

    for witness in witnesses {
        let divergence = detect_divergence::<Sha256>(
            Some(primary),
            witness,
            primary_trace.clone().into_vec(),
            max_clock_drift,
            max_block_lag,
        )
        .await;

        let evidence = match divergence {
            Ok(Some(divergence)) => divergence.evidence,
            Ok(None) => {
                info!(
                    "no divergence found between primary and witness {}",
                    witness.peer_id()
                );

                continue;
            }
            Err(e) => {
                error!(
                    "failed to run attack detector against witness {}: {e}",
                    witness.peer_id()
                );

                continue;
            }
        };

        // Report the evidence to the witness
        witness
            .report_evidence(Evidence::from(evidence.against_primary))
            .await
            .map_err(|e| eyre!("failed to report evidence to witness: {}", e))?;

        if let Some(against_witness) = evidence.against_witness {
            // Report the evidence to the primary
            primary
                .report_evidence(Evidence::from(against_witness))
                .await
                .map_err(|e| eyre!("failed to report evidence to primary: {}", e))?;
        }
    }

    Ok(())
}

async fn make_provider(
    chain_id: &str,
    rpc_addr: HttpClientUrl,
    trusted_height: Height,
    trusted_hash: Hash,
    options: Options,
) -> Result<Provider> {
    use tendermint_rpc::client::CompatMode;

    let rpc_client = HttpClient::builder(rpc_addr)
        .compat_mode(CompatMode::V0_34)
        .build()?;

    let node_id = rpc_client.status().await?.node_info.id;
    let light_store = Box::new(MemoryStore::new());

    let instance =
        LightClientBuilder::prod(node_id, rpc_client.clone(), light_store, options, None)
            .trust_primary_at(trusted_height, trusted_hash)?
            .build();

    Ok(Provider::new(chain_id.to_string(), instance, rpc_client))
}
