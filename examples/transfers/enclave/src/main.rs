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
    collections::HashMap,
    marker::PhantomData,
    sync::{Arc, Mutex},
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
        handler::Handler,
        key_manager::KeyManager,
        kv_store::{ConfigKey, ContractKey, KvStore, NonceKey, TypedStore},
        server::{QuartzServer, WsListenerConfig},
        Enclave,
    },
    proto::{
        core_server::{Core, CoreServer},
        InstantiateRequest, InstantiateResponse, SessionCreateRequest, SessionCreateResponse,
        SessionSetPubKeyRequest, SessionSetPubKeyResponse,
    },
};
use tokio::sync::{mpsc, oneshot, RwLock};
use tonic::{transport::Server, Request, Response, Status};
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

#[async_trait::async_trait]
impl KeyManager for DefaultKeyManager {
    type PubKey = VerifyingKey;

    async fn keygen(&mut self) {
        self.sk = Some(SigningKey::random(&mut rand::thread_rng()));
    }

    async fn pub_key(&self) -> Option<Self::PubKey> {
        self.sk.clone().map(|sk| VerifyingKey::from(&sk))
    }
}

#[derive(Clone, Debug, Default)]
struct DefaultKvStore {
    config: Option<Config>,
    contract: Option<AccountId>,
    nonce: Option<Nonce>,
}

#[derive(Debug, Display)]
enum StoreError {}

#[async_trait::async_trait]
impl KvStore<ContractKey<AccountId>, AccountId> for DefaultKvStore {
    type Error = StoreError;

    async fn set(
        &mut self,
        _key: ContractKey<AccountId>,
        value: AccountId,
    ) -> Result<Option<AccountId>, Self::Error> {
        Ok(self.contract.replace(value))
    }

    async fn get(&self, _key: ContractKey<AccountId>) -> Result<Option<AccountId>, Self::Error> {
        Ok(self.contract.clone().take())
    }

    async fn delete(&mut self, _key: ContractKey<AccountId>) -> Result<(), Self::Error> {
        unimplemented!()
    }
}

#[async_trait::async_trait]
impl KvStore<NonceKey, Nonce> for DefaultKvStore {
    type Error = StoreError;

    async fn set(&mut self, _key: NonceKey, value: Nonce) -> Result<Option<Nonce>, Self::Error> {
        Ok(self.nonce.replace(value))
    }

    async fn get(&self, _key: NonceKey) -> Result<Option<Nonce>, Self::Error> {
        Ok(self.nonce.clone().take())
    }

    async fn delete(&mut self, _key: NonceKey) -> Result<(), Self::Error> {
        unimplemented!()
    }
}

#[async_trait::async_trait]
impl KvStore<ConfigKey, Config> for DefaultKvStore {
    type Error = StoreError;

    async fn set(&mut self, _key: ConfigKey, value: Config) -> Result<Option<Config>, Self::Error> {
        Ok(self.config.replace(value))
    }

    async fn get(&self, _key: ConfigKey) -> Result<Option<Config>, Self::Error> {
        Ok(self.config.clone().take())
    }

    async fn delete(&mut self, _key: ConfigKey) -> Result<(), Self::Error> {
        unimplemented!()
    }
}

#[derive(Clone, Debug, Default)]
struct BincodeKvStore {
    map: HashMap<String, Vec<u8>>,
}

#[derive(Debug, Display)]
enum BincodeError {
    /// encode error: {0}
    Encode(bincode::error::EncodeError),
    /// decode error: {0}
    Decode(bincode::error::DecodeError),
}

#[async_trait::async_trait]
impl<K, V> KvStore<K, V> for BincodeKvStore
where
    K: ToString + Send + Sync + 'static,
    V: bincode::Encode + bincode::Decode + Send + Sync + 'static,
{
    type Error = BincodeError;

    async fn set(&mut self, key: K, value: V) -> Result<Option<V>, Self::Error> {
        let key = key.to_string();
        let value = bincode::encode_to_vec(&value, bincode::config::standard())
            .map_err(BincodeError::Encode)?;
        let prev_value = self
            .map
            .insert(key, value)
            .map(|v| bincode::decode_from_slice(&v, bincode::config::standard()))
            .transpose()
            .map_err(BincodeError::Decode)?
            .map(|(v, _)| v);
        Ok(prev_value)
    }

    async fn get(&self, key: K) -> Result<Option<V>, Self::Error> {
        let key = key.to_string();
        Ok(self
            .map
            .get(&key)
            .map(|v| bincode::decode_from_slice(&v, bincode::config::standard()))
            .transpose()
            .map_err(BincodeError::Decode)?
            .map(|(v, _)| v))
    }

    async fn delete(&mut self, key: K) -> Result<(), Self::Error> {
        let key = key.to_string();
        let _ = self.map.remove(&key);
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct DefaultEnclave<
    A = DefaultAttestor,
    C = DefaultChainClient,
    K = DefaultKeyManager,
    S = DefaultKvStore,
> {
    attestor: A,
    chain_client: C,
    key_manager: K,
    store: S,
}

#[derive(Clone, Debug)]
struct SharedKeyManager<K> {
    inner: Arc<RwLock<K>>,
}

#[async_trait::async_trait]
impl<K: KeyManager> KeyManager for SharedKeyManager<K> {
    type PubKey = K::PubKey;

    async fn keygen(&mut self) {
        self.inner.write().await.keygen().await
    }

    async fn pub_key(&self) -> Option<Self::PubKey> {
        self.inner.read().await.pub_key().await
    }
}

#[derive(Clone, Debug)]
struct SharedKvStore<S> {
    inner: Arc<RwLock<S>>,
}

#[async_trait::async_trait]
impl<S, K, V> KvStore<K, V> for SharedKvStore<S>
where
    S: KvStore<K, V>,
    K: Send + Sync + 'static,
    V: Send + Sync + 'static,
{
    type Error = S::Error;

    async fn set(&mut self, key: K, value: V) -> Result<Option<V>, Self::Error> {
        self.inner.write().await.set(key, value).await
    }

    async fn get(&self, key: K) -> Result<Option<V>, Self::Error> {
        self.inner.read().await.get(key).await
    }

    async fn delete(&mut self, key: K) -> Result<(), Self::Error> {
        self.inner.write().await.delete(key).await
    }
}

#[derive(Clone, Debug)]
enum KvStoreAction<K, V> {
    Set(K, V),
    Get(K),
    Delete(K),
}

#[derive(Clone, Debug)]
enum KvStoreActionResult<V> {
    Set(Option<V>),
    Get(Option<V>),
    Delete,
    Err,
}

#[derive(Debug)]
struct KvStoreRequest<K, V> {
    action: KvStoreAction<K, V>,
    resp_tx: oneshot::Sender<KvStoreActionResult<V>>,
}

#[derive(Clone, Debug)]
struct MpscKvStore<S, K, V> {
    req_tx: mpsc::Sender<KvStoreRequest<K, V>>,
    _phantom: PhantomData<S>,
}

#[async_trait::async_trait]
impl<S, K, V> KvStore<K, V> for MpscKvStore<S, K, V>
where
    S: KvStore<K, V>,
    K: Send + Sync + 'static,
    V: Send + Sync + 'static,
{
    type Error = S::Error;

    async fn set(&mut self, key: K, value: V) -> Result<Option<V>, Self::Error> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.req_tx
            .send(KvStoreRequest {
                action: KvStoreAction::Set(key, value),
                resp_tx,
            })
            .await
            .expect("KvStoreRequest channel closed");

        if let KvStoreActionResult::Set(prev_v) = resp_rx.await.expect("resp_rx") {
            Ok(prev_v)
        } else {
            unreachable!()
        }
    }

    async fn get(&self, key: K) -> Result<Option<V>, Self::Error> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.req_tx
            .send(KvStoreRequest {
                action: KvStoreAction::Get(key),
                resp_tx,
            })
            .await
            .expect("KvStoreRequest channel closed");

        if let KvStoreActionResult::Get(prev_v) = resp_rx.await.expect("resp_rx") {
            Ok(prev_v)
        } else {
            unreachable!()
        }
    }

    async fn delete(&mut self, key: K) -> Result<(), Self::Error> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.req_tx
            .send(KvStoreRequest {
                action: KvStoreAction::Delete(key),
                resp_tx,
            })
            .await
            .expect("KvStoreRequest channel closed");

        if let KvStoreActionResult::Delete = resp_rx.await.expect("resp_rx") {
            Ok(())
        } else {
            unreachable!()
        }
    }
}

#[async_trait::async_trait]
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
    type KeyManager = K;
    type Store = S;

    async fn attestor(&self) -> Self::Attestor {
        self.attestor.clone()
    }

    async fn chain_client(&self) -> Self::ChainClient {
        self.chain_client.clone()
    }

    async fn key_manager(&self) -> Self::KeyManager {
        self.key_manager.clone()
    }

    async fn store(&self) -> Self::Store {
        self.store.clone()
    }
}

#[async_trait::async_trait]
impl<A, C, K, S> Core for DefaultEnclave<A, C, K, S>
where
    A: Attestor + Clone,
    C: ChainClient<Contract = AccountId> + Clone,
    K: KeyManager<PubKey = VerifyingKey> + Clone,
    S: TypedStore<ContractKey<AccountId>> + TypedStore<NonceKey> + TypedStore<ConfigKey> + Clone,
{
    async fn instantiate(
        &self,
        request: Request<InstantiateRequest>,
    ) -> Result<Response<InstantiateResponse>, Status> {
        request.handle(self).await
    }

    async fn session_create(
        &self,
        request: Request<SessionCreateRequest>,
    ) -> Result<Response<SessionCreateResponse>, Status> {
        request.handle(self).await
    }

    async fn session_set_pub_key(
        &self,
        request: Request<SessionSetPubKeyRequest>,
    ) -> Result<Response<SessionSetPubKeyResponse>, Status> {
        request.handle(self).await
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::sleep;

    use super::*;

    #[tokio::test]
    async fn test_tonic_service() -> Result<(), Box<dyn std::error::Error>> {
        let enclave = DefaultEnclave {
            attestor: attestor::MockAttestor::default(),
            chain_client: DefaultChainClient::default(),
            key_manager: SharedKeyManager {
                inner: Arc::new(RwLock::new(DefaultKeyManager::default())),
            },
            store: SharedKvStore {
                inner: Arc::new(RwLock::new(DefaultKvStore::default())),
            },
        };
        let addr = "127.0.0.1:9095".parse().expect("hardcoded correct ip");

        Server::builder()
            .add_service(CoreServer::new(enclave))
            .serve_with_shutdown(addr, async {
                sleep(Duration::from_secs(1)).await;
                println!("Shutting down...");
            })
            .await?;

        Ok(())
    }
}
