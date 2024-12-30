#![doc = include_str!("../README.md")]
// #![forbid(unsafe_code)]
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

use quartz_contract_core::state::Config;
use serde::de::DeserializeOwned;

use crate::{
    attestor::Attestor,
    chain_client::ChainClient,
    kv_store::{ContractKey, NonceKey, TypedStore},
    signer::Signer,
};

pub mod attestor;
pub mod chain_client;
pub mod error;
pub mod handler;
pub mod kv_store;
pub mod server;
pub mod signer;
pub mod types;

pub trait Enclave {
    type Attestor: Attestor;
    type ChainClient: ChainClient<Contract = Self::Contract>;
    type Contract: DeserializeOwned + Clone + ToString;
    type Signer: Signer;
    type Store: TypedStore<ContractKey<Self::Contract>> + TypedStore<NonceKey>;

    fn attestor(&self) -> Self::Attestor;
    fn config(&self) -> Config;
    fn chain_client(&self) -> Self::ChainClient;
    fn signer(&self) -> Self::Signer;
    fn store(&self) -> Self::Store;
}
