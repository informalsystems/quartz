use crate::{handler::Handler, Enclave};

pub type HostResult<R, E> = Result<<R as Handler<E>>::Response, <R as Handler<E>>::Error>;

#[async_trait::async_trait]
pub trait Host {
    type Enclave: Enclave;

    async fn enclave_call<R>(&self, request: R) -> HostResult<R, Self::Enclave>
    where
        R: Handler<Self::Enclave>;
}
