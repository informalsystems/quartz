use std::{num::ParseIntError, str::FromStr};

use clap::Parser;
use color_eyre::eyre::{eyre, Result};
use cosmrs::AccountId;
use quartz_cw_proof::proof::cw::RawCwProof;
use serde::{Deserialize, Serialize};
use tendermint_light_client::types::{Hash, Height, LightBlock, TrustThreshold};
use tendermint_rpc::HttpClientUrl;
use tracing::metadata::LevelFilter;

pub fn parse_trust_threshold(s: &str) -> Result<TrustThreshold> {
    if let Some((l, r)) = s.split_once('/') {
        TrustThreshold::new(l.parse()?, r.parse()?).map_err(Into::into)
    } else {
        Err(eyre!(
            "invalid trust threshold: {s}, format must be X/Y where X and Y are integers"
        ))
    }
}

#[derive(Clone, Debug)]
pub struct List<T>(pub Vec<T>);

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
pub struct Verbosity {
    /// Increase verbosity, can be repeated up to 2 times
    #[arg(long, short, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

impl Verbosity {
    pub fn to_level_filter(&self) -> LevelFilter {
        match self.verbose {
            0 => LevelFilter::INFO,
            1 => LevelFilter::DEBUG,
            _ => LevelFilter::TRACE,
        }
    }

    fn default() -> Self {
        Self { verbose: 0 }
    }
}

impl FromStr for Verbosity {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let verbose: u8 = s.parse()?;
        Ok(Self { verbose })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProofOutput {
    pub light_client_proof: Vec<LightBlock>,
    pub merkle_proof: RawCwProof,
}

// TODO: Investigate if it's possible to derive default using Clap's default values, or otherwise find better default values
impl Default for Config {
    fn default() -> Self {
        Config {
            chain_id: String::default(),
            primary: "http://127.0.0.1:26657".parse().unwrap(),
            witnesses: "http://127.0.0.1:26657".parse().unwrap(),
            trusted_height: Height::default(),
            trusted_hash: Hash::default(),
            trust_threshold: TrustThreshold::TWO_THIRDS,
            trusting_period: 1209600u64,
            max_clock_drift: 5u64,
            max_block_lag: 5u64,
            verbose: Verbosity::default(),
            contract_address: "wasm14qdftsfk6fwn40l0xmruga08xlczl4g05npy70"
                .parse()
                .unwrap(),
            storage_key: String::default(),
        }
    }
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// Identifier of the chain
    #[clap(long)]
    pub chain_id: String,

    /// Primary RPC address
    #[clap(long, default_value = "http://127.0.0.1:26657")]
    pub primary: HttpClientUrl,

    /// Comma-separated list of witnesses RPC addresses
    #[clap(long)]
    pub witnesses: List<HttpClientUrl>,

    /// Height of trusted header
    #[clap(long)]
    pub trusted_height: Height,

    /// Hash of trusted header
    #[clap(long)]
    pub trusted_hash: Hash,

    /// Trust threshold
    #[clap(long, value_parser = parse_trust_threshold, default_value_t = TrustThreshold::TWO_THIRDS)]
    pub trust_threshold: TrustThreshold,

    /// Trusting period, in seconds (default: two weeks)
    #[clap(long, default_value = "1209600")]
    pub trusting_period: u64,

    /// Maximum clock drift, in seconds
    #[clap(long, default_value = "5")]
    pub max_clock_drift: u64,

    /// Maximum block lag, in seconds
    #[clap(long, default_value = "5")]
    pub max_block_lag: u64,

    /// Increase verbosity
    #[clap(flatten)]
    pub verbose: Verbosity,

    /// Address of the CosmWasm contract
    #[clap(long)]
    pub contract_address: AccountId,

    /// Storage key of the state item for which proofs must be retrieved
    #[clap(long)]
    pub storage_key: String,
}
