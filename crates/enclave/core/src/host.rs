use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
    net::SocketAddr,
    path::PathBuf,
};

use anyhow::anyhow;
use cosmrs::AccountId;
use futures_util::StreamExt;
use log::{error, info, trace, warn};
use quartz_proto::quartz::core_server::{Core, CoreServer};
use reqwest::Url;
use serde::Serialize;
use tendermint_rpc::{
    event::Event as TmEvent,
    query::{EventType, Query},
    SubscriptionClient, WebSocketClient,
};
use tokio::sync::mpsc::Receiver;
use tonic::{transport::Server, Status};

use crate::{
    backup_restore::Backup,
    chain_client::{default::DefaultChainClient, ChainClient},
    event::QuartzEvent,
    handler::Handler,
    store::Store,
    Enclave, Notification,
};

pub type Response<R, E> = <R as Handler<E>>::Response;

/// The `Host` trait defines the untrusted side of the Quartz framework,
/// acting as the gateway between the blockchain and the trusted enclave.
///
/// The host is responsible for:
/// - Listening for blockchain events via a [`ChainClient`].
/// - Constructing enclave requests from events by using an event handler.
/// - Forwarding these requests to the enclave through [`Host::enclave_call`].
/// - Relaying responses from the enclave back to the blockchain (typically by sending a transaction).
///
/// This separation ensures that all communication with the enclave is derived from
/// on-chain data, thereby providing replay protection and a secure communication channel.
#[async_trait::async_trait]
pub trait Host: Send + Sync + 'static + Sized {
    /// The blockchain client type for interacting with on-chain data.
    type ChainClient: ChainClient;
    /// The trusted enclave type that processes requests.
    type Enclave: Enclave;
    /// The error type used for reporting host-level errors.
    type Error: Send + Sync + 'static;
    /// The event type that, when handled, produces an enclave request.
    ///
    /// The associated response from handling an event must be of type `Self::Request`.
    type Event: Handler<Self::ChainClient, Response = Self::Request>;
    /// The enclave request type that is processed by the enclave.
    ///
    /// This type must implement [`Handler`] for `Self::Enclave`.
    type Request: Handler<Self::Enclave>;

    /// Forwards an enclave request to the trusted enclave and returns the corresponding response.
    ///
    /// # Parameters
    ///
    /// - `request`: An enclave request to be processed by the enclave.
    ///
    /// # Returns
    ///
    /// A `Result` containing a [`Response`] that wraps the enclave's output, or an error of type `Self::Error`.
    async fn enclave_call(
        &self,
        request: Self::Request,
    ) -> Result<Response<Self::Request, Self::Enclave>, Self::Error>;

    /// Consumes the host and starts an event loop by connecting to the specified URL,
    /// with an optional query filter for events.
    ///
    /// The host listens for blockchain events, transforms them into enclave requests,
    /// forwards these requests to the enclave, and handles the responses.
    ///
    /// # Parameters
    ///
    /// - `url`: The URL of the blockchain event endpoint (typically a WebSocket endpoint).
    /// - `query`: An optional filter for subscribing to specific blockchain events.
    ///
    /// # Returns
    ///
    /// A `Result` indicating successful processing of events (`Ok(())`) or an error (`Err(Self::Error)`).
    async fn serve_with_query(
        self,
        url: Url,
        rpc_addr: SocketAddr,
        query: Option<Query>,
    ) -> Result<(), Self::Error>;

    /// A convenience method that starts the event loop without a specific query filter.
    async fn serve(self, url: Url, rpc_addr: SocketAddr) -> Result<(), Self::Error> {
        self.serve_with_query(url, rpc_addr, None).await
    }
}

/// The default generic implementation of the untrusted host in the Quartz framework.
///
/// `DefaultHost` ties together the essential components needed to bridge on-chain events with
/// enclave requests. It encapsulates an enclave instance, a blockchain client, and a gas
/// configuration function used to compute transaction settings (such as gas fees) for enclave responses.
///
/// This host implementation is responsible for:
/// - Listening for blockchain events and converting them into enclave requests.
/// - Forwarding these requests to the enclave via an asynchronous call.
/// - Sending the enclave’s response back to the blockchain using the provided chain client.
///
/// ### Note
/// This implementation consumes an `Enclave` instance and calls it directly. Therefore, it is
/// expected to be run inside a TEE.
#[derive(Debug)]
pub struct DefaultHost<R, EV, GF, E, C = DefaultChainClient> {
    enclave: E,
    chain_client: C,
    gas_fn: GF,
    notifier_rx: Receiver<Notification>,
    _phantom: PhantomData<(R, EV)>,
}

impl<R, EV, GF, E, C> DefaultHost<R, EV, GF, E, C>
where
    R: Handler<E>,
    C: ChainClient,
{
    pub fn new(
        enclave: E,
        chain_client: C,
        gas_fn: GF,
        notifier_rx: Receiver<Notification>,
    ) -> Self {
        Self {
            enclave,
            chain_client,
            gas_fn,
            notifier_rx,
            _phantom: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl<R, EV, GF, E, C> Host for DefaultHost<R, EV, GF, E, C>
where
    E: Enclave + Backup<Config = PathBuf, Error = anyhow::Error> + Clone + Core,
    <E as Enclave>::Store: Store<Contract = AccountId>,
    C: ChainClient<Contract = AccountId, Error = anyhow::Error>,
    <C as ChainClient>::TxOutput: Display,
    R: Handler<E, Error = Status> + Debug,
    <R as Handler<E>>::Response: Serialize + Send + Sync + 'static,
    EV: Handler<C, Response = R, Error = anyhow::Error>,
    EV: TryFrom<TmEvent, Error = anyhow::Error>,
    GF: Fn(&<R as Handler<E>>::Response) -> <C as ChainClient>::TxConfig + Send + Sync + 'static,
{
    type ChainClient = C;
    type Enclave = E;
    type Error = anyhow::Error;
    type Event = QuartzEvent<EV>;
    type Request = R;

    async fn enclave_call(
        &self,
        request: Self::Request,
    ) -> Result<Response<Self::Request, Self::Enclave>, Self::Error> {
        // call the enclave directly with the request
        request
            .handle(&self.enclave)
            .await
            .map_err(|e| anyhow!("enclave call failed: {}", e))
    }

    async fn serve_with_query(
        mut self,
        url: Url,
        rpc_addr: SocketAddr,
        query: Option<Query>,
    ) -> Result<(), Self::Error> {
        // connect to the websocket client
        let (client, driver) = WebSocketClient::new(url.as_str()).await.unwrap();
        let driver_handle = tokio::spawn(async move { driver.run().await });

        let restore_err = self.enclave.try_restore(PathBuf::default()).await.is_err();
        if restore_err {
            // run handshake if restore failed (i.e. this is a fresh start)
            let enclave = self.enclave.clone();
            tokio::spawn(async move {
                Server::builder()
                    .add_service(CoreServer::new(enclave))
                    .serve(rpc_addr)
                    .await
            });
        }

        // wait for handshake
        if let Some(Notification::HandshakeComplete) = self.notifier_rx.recv().await {
            // FIXME(hu55a1n1): need configurable path
            self.enclave.backup(PathBuf::default()).await?;
        }

        // subscribe to relevant events
        // TODO: default to `Query::from(EventType::Tx).and_eq("wasm._contract_address", contract)`
        let query = query.unwrap_or(Query::from(EventType::Tx));
        let mut subs = client.subscribe(query).await.unwrap();

        // wait and listen for events
        while let Some(Ok(event)) = subs.next().await {
            trace!("Received event: {event:?}");

            // attempt to decode event if relevant
            let event = match Self::Event::try_from(event) {
                Ok(e) => e,
                Err(e) => {
                    trace!("Failed to decode event: {e}");
                    continue;
                }
            };

            // Make sure the contract in the event is the same as the paired contract.
            // This check is not really required since the proof-of-publication check will check
            // if there is a mismatch anyway, but it allows us to short-circuit here.
            let contract = event.contract.clone();
            let expected_contract = self
                .enclave
                .store()
                .await
                .get_contract()
                .await
                .map_err(|_| anyhow!("contract read failure"))?
                .expect("contract must be set");
            if contract != expected_contract {
                error!("contract != expected_contract");
                continue;
            }

            // handle event (through event handler) and generate enclave request
            let request = match event.handle(&self.chain_client).await {
                Ok(r) => r,
                Err(e) => {
                    warn!("event handler: {e}");
                    continue;
                }
            };

            trace!("Handling request: {request:?}");

            // call enclave with request and get response
            let response = match self.enclave_call(request).await {
                Ok(r) => r,
                Err(e) => {
                    warn!("request handler: {e}");
                    continue;
                }
            };

            // submit response to the chain
            let tx_config = (self.gas_fn)(&response);
            let output = self
                .chain_client
                .send_tx(&contract, response, tx_config)
                .await;
            match output {
                Ok(o) => info!("tx output: {o}"),
                Err(e) => warn!("send_tx: {e}"),
            }
        }

        client.close().expect("Failed to close client");
        let _ = driver_handle.await;

        Ok(())
    }
}
