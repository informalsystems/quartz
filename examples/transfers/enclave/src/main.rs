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
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};

use clap::Parser;
use cli::Cli;
use cosmrs::AccountId;
use displaydoc::Display;
use k256::ecdsa::{SigningKey, VerifyingKey};
use quartz_common::{
    contract::state::{Config, LightClientOpts, Nonce},
    enclave::{
        attestor::{self, Attestor, DefaultAttestor},
        chain_client::ChainClient,
        key_manager::KeyManager,
        kv_store::{ConfigKey, ContractKey, KvStore, NonceKey, TypedStore},
        server::{QuartzServer, WsListenerConfig},
        Enclave,
    },
};
use tokio::sync::mpsc;
use transfers_server::{TransfersOp, TransfersService};

use crate::wslistener::WsListener;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let admin_sk = std::env::var("ADMIN_SK")
        .map_err(|_| anyhow::anyhow!("Admin secret key not found in env vars"))?;

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
    let attestor = attestor::DcapAttestor {
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
        ws_url: args.ws_url,
        grpc_url: args.grpc_url,
        tx_sender: args.tx_sender,
        trusted_hash: args.trusted_hash,
        trusted_height: args.trusted_height,
        chain_id: args.chain_id,
        admin_sk,
    };

    // Event queue
    let (tx, mut rx) = mpsc::channel::<TransfersOp<DefaultAttestor>>(1);
    // Consumer task: dequeue and process events
    tokio::spawn(async move {
        while let Some(op) = rx.recv().await {
            if let Err(e) = op.client.process(op.event, op.config).await {
                println!("Error processing queued event: {}", e);
            }
        }
    });

    let contract = Arc::new(Mutex::new(None));
    let sk = Arc::new(Mutex::new(None));

    QuartzServer::new(
        config.clone(),
        contract.clone(),
        sk.clone(),
        attestor.clone(),
        ws_config,
    )
    .add_service(TransfersService::new(config, sk, contract, attestor, tx))
    .serve(args.rpc_addr)
    .await?;

    Ok(())
}

#[derive(Clone, Debug, Default)]
struct DefaultChainClient;

impl ChainClient for DefaultChainClient {
    const CHAIN_ID: &'static str = "pion-1";
    type Contract = AccountId;
}

#[derive(Clone, Default)]
struct DefaultKeyManager {
    sk: Option<SigningKey>,
}

impl KeyManager for DefaultKeyManager {
    type PubKey = VerifyingKey;

    fn keygen(&mut self) {
        self.sk = Some(SigningKey::random(&mut rand::thread_rng()));
    }

    fn pub_key(&self) -> Option<Self::PubKey> {
        self.sk.clone().map(|sk| VerifyingKey::from(&sk))
    }
}

#[derive(Clone, Debug, Default)]
struct DefaultStore {
    config: Option<Config>,
    contract: Option<AccountId>,
    nonce: Option<Nonce>,
}

#[derive(Debug, Display)]
enum StoreError {}

impl KvStore<ContractKey<AccountId>, AccountId> for DefaultStore {
    type Error = StoreError;

    fn set(
        &mut self,
        _key: ContractKey<AccountId>,
        value: AccountId,
    ) -> Result<Option<AccountId>, Self::Error> {
        Ok(self.contract.replace(value))
    }

    fn get(&self, _key: ContractKey<AccountId>) -> Result<Option<AccountId>, Self::Error> {
        Ok(self.contract.clone().take())
    }

    fn delete(&mut self, _key: ContractKey<AccountId>) -> Result<(), Self::Error> {
        unimplemented!()
    }
}

impl KvStore<NonceKey, Nonce> for DefaultStore {
    type Error = StoreError;

    fn set(&mut self, _key: NonceKey, value: Nonce) -> Result<Option<Nonce>, Self::Error> {
        Ok(self.nonce.replace(value))
    }

    fn get(&self, _key: NonceKey) -> Result<Option<Nonce>, Self::Error> {
        Ok(self.nonce.clone().take())
    }

    fn delete(&mut self, _key: NonceKey) -> Result<(), Self::Error> {
        unimplemented!()
    }
}

impl KvStore<ConfigKey, Config> for DefaultStore {
    type Error = StoreError;

    fn set(&mut self, _key: ConfigKey, value: Config) -> Result<Option<Config>, Self::Error> {
        Ok(self.config.replace(value))
    }

    fn get(&self, _key: ConfigKey) -> Result<Option<Config>, Self::Error> {
        Ok(self.config.clone().take())
    }

    fn delete(&mut self, _key: ConfigKey) -> Result<(), Self::Error> {
        unimplemented!()
    }
}

#[derive(Clone, Debug)]
struct DefaultEnclave<
    A = DefaultAttestor,
    C = DefaultChainClient,
    K = DefaultKeyManager,
    S = DefaultStore,
> {
    attestor: A,
    chain_client: C,
    key_manager: SharedKeyManager<K>,
    store: SharedKvStore<S>,
}

#[derive(Clone, Debug)]
struct SharedKeyManager<K> {
    inner: Arc<RwLock<K>>,
}

impl<K: KeyManager> KeyManager for SharedKeyManager<K> {
    type PubKey = K::PubKey;

    fn keygen(&mut self) {
        self.inner
            .write()
            .expect("shared key-manager write error")
            .keygen()
    }

    fn pub_key(&self) -> Option<Self::PubKey> {
        self.inner
            .read()
            .expect("shared key-manager read error")
            .pub_key()
    }
}

#[derive(Clone, Debug)]
struct SharedKvStore<S> {
    inner: Arc<RwLock<S>>,
}

impl<S, K, V> KvStore<K, V> for SharedKvStore<S>
where
    S: KvStore<K, V>,
{
    type Error = S::Error;

    fn set(&mut self, key: K, value: V) -> Result<Option<V>, Self::Error> {
        self.inner
            .write()
            .expect("shared kv-store write error")
            .set(key, value)
    }

    fn get(&self, key: K) -> Result<Option<V>, Self::Error> {
        self.inner
            .read()
            .expect("shared kv-store read error")
            .get(key)
    }

    fn delete(&mut self, key: K) -> Result<(), Self::Error> {
        self.inner
            .write()
            .expect("shared kv-store write error")
            .delete(key)
    }
}

impl<A, C, K, S> Enclave for DefaultEnclave<A, C, K, S>
where
    A: Attestor + Clone,
    C: ChainClient<Contract = AccountId> + Clone,
    K: KeyManager + Clone,
    S: TypedStore<ContractKey<AccountId>> + TypedStore<NonceKey> + TypedStore<ConfigKey> + Clone,
{
    type Attestor = A;
    type ChainClient = C;
    type Contract = AccountId;
    type KeyManager = SharedKeyManager<K>;
    type Store = SharedKvStore<S>;

    fn attestor(&self) -> Self::Attestor {
        self.attestor.clone()
    }

    fn chain_client(&self) -> Self::ChainClient {
        self.chain_client.clone()
    }

    fn key_manager(&self) -> Self::KeyManager {
        self.key_manager.clone()
    }

    fn store(&self) -> Self::Store {
        self.store.clone()
    }
}
