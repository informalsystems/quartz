use std::marker::PhantomData;

use futures_util::StreamExt;
use reqwest::Url;
use tendermint_rpc::{
    event::Event as TmEvent,
    query::{EventType, Query},
    SubscriptionClient, WebSocketClient,
};
use tonic::Status;

use crate::{chain_client::ChainClient, handler::Handler, Enclave};

pub type Response<R, E> = <R as Handler<E>>::Response;

#[async_trait::async_trait]
pub trait Host: Send + Sync + 'static + Sized {
    type ChainClient: ChainClient;
    type Enclave: Enclave;
    type Error: Send + Sync + 'static;
    type Event: Handler<Self::ChainClient, Response = Self::Request>;
    type Request: Handler<Self::Enclave>;

    async fn enclave_call(
        &self,
        request: Self::Request,
    ) -> Result<Response<Self::Request, Self::Enclave>, Self::Error>;

    async fn serve(self, url: Url) -> Result<(), Self::Error>;
}

#[derive(Clone, Debug)]
pub struct DefaultHost<E, C, R, EV> {
    enclave: E,
    chain_client: C,
    _phantom: PhantomData<(R, EV)>,
}

impl<E, C, R, EV> DefaultHost<E, C, R, EV> {
    pub fn new(enclave: E, chain_client: C) -> Self {
        Self {
            enclave,
            chain_client,
            _phantom: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl<E, C, R, EV> Host for DefaultHost<E, C, R, EV>
where
    E: Enclave,
    C: ChainClient,
    R: Handler<E, Error = Status>,
    EV: Handler<C, Response = R, Error = Status>,
    EV: TryFrom<TmEvent, Error = ()>,
    <EV as TryFrom<TmEvent>>::Error: Send + Sync + 'static,
{
    type ChainClient = C;
    type Enclave = E;
    type Error = Status;
    type Event = EV;
    type Request = R;

    async fn enclave_call(
        &self,
        request: Self::Request,
    ) -> Result<Response<Self::Request, Self::Enclave>, Self::Error> {
        request.handle(&self.enclave).await
    }

    async fn serve(self, url: Url) -> Result<(), Self::Error> {
        let (client, driver) = WebSocketClient::new(url.as_str()).await.unwrap();

        let driver_handle = tokio::spawn(async move { driver.run().await });

        let mut subs = client.subscribe(Query::from(EventType::Tx)).await.unwrap();
        while let Some(Ok(event)) = subs.next().await {
            if let Ok(event) = Self::Event::try_from(event) {
                let request = event.handle(&self.chain_client).await?;
                let _response = self.enclave_call(request).await?;

                // TODO: send response to chain (in tx)
            }
        }

        client.close().expect("Failed to close client");
        let _ = driver_handle.await;

        Ok(())
    }
}
