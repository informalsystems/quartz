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
    time::Duration,
};

use color_eyre::{
    eyre::{eyre, Result},
    Report,
};
use cw_proof::{
    error::ProofError,
    proof::{cw::CwProof, key::CwAbciKey, Proof},
};
use futures::future::join_all;
use tendermint::{crypto::default::Sha256, evidence::Evidence, Hash};
use tendermint_light_client::{
    builder::LightClientBuilder,
    light_client::Options,
    store::memory::MemoryStore,
    types::{Height, LightBlock},
};
use tendermint_light_client_detector::{detect_divergence, Error, Provider, Trace};
use tendermint_rpc::{client::HttpClient, Client, HttpClientUrl};
use tracing::{error, info};

const WASM_STORE_KEY: &str = "/store/wasm/key";

use crate::config::{Config as TmProverConfig, ProofOutput};

pub async fn prove(
    TmProverConfig {
        chain_id,
        primary,
        witnesses,
        trusted_height,
        trusted_hash,
        trust_threshold,
        trusting_period,
        max_clock_drift,
        max_block_lag,
        trace_file,
        verbose: _,
        contract_address,
        storage_key,
    }: TmProverConfig,
) -> Result<()> {
    let options = Options {
        trust_threshold,
        trusting_period: Duration::from_secs(trusting_period),
        clock_drift: Duration::from_secs(max_clock_drift),
    };

    let mut provider = make_provider(
        &chain_id,
        primary.clone(),
        trusted_height,
        trusted_hash,
        options,
    )
    .await?;

    let client = HttpClient::builder(primary.clone()).build()?;

    let trusted_block = provider
        .latest_trusted()
        .ok_or_else(|| eyre!("No trusted state found for primary"))?;

    info!("Getting status of node");
    let status = client.status().await?;
    let latest_height = status.sync_info.latest_block_height;
    let latest_app_hash = status.sync_info.latest_app_hash;

    // `proof_height` is the height at which we want to query the blockchain's state
    // This is one less than than the `latest_height` because we want to verify the merkle-proof for
    // the state against the `app_hash` at `latest_height`.
    // (because Tendermint commits to the latest `app_hash` in the subsequent block)
    let proof_height = (latest_height.value() - 1)
        .try_into()
        .expect("infallible conversion");

    info!("Verifying to latest height on primary...");

    let primary_block = provider.verify_to_height(latest_height)?;

    info!("Verified to height {} on primary", primary_block.height());
    let mut primary_trace = provider.get_trace(primary_block.height());
    let witnesses = join_all(witnesses.0.into_iter().map(|addr: HttpClientUrl| {
        make_provider(
            &chain_id,
            addr,
            trusted_block.height(),
            trusted_block.signed_header.header.hash(),
            options,
        )
    }))
    .await;

    let mut witnesses = witnesses.into_iter().collect::<Result<Vec<_>>>()?;

    let max_clock_drift = Duration::from_secs(max_clock_drift);
    let max_block_lag = Duration::from_secs(max_block_lag);

    run_detector(
        &mut provider,
        witnesses.as_mut_slice(),
        primary_trace.clone(),
        max_clock_drift,
        max_block_lag,
    )
    .await?;

    let path = WASM_STORE_KEY.to_owned();
    let data = CwAbciKey::new(contract_address, storage_key, None);
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

    if let Some(trace_file) = trace_file {
        // replace the last block in the trace (i.e. the (latest - 1) block) with the latest block
        // we don't actually verify the latest block because it will be verified on the other side
        let latest_block = provider.fetch_light_block(latest_height)?;
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
