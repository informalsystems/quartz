use std::marker::PhantomData;

use crate::{handler::Handler, Enclave};

pub type HostResult<R, E> = Result<<R as Handler<E>>::Response, <R as Handler<E>>::Error>;

#[async_trait::async_trait]
pub trait Host: Send + Sync + 'static {
    type Enclave: Enclave;
    type Request: Handler<Self::Enclave>;

    async fn enclave_call(
        &self,
        request: Self::Request,
    ) -> HostResult<Self::Request, Self::Enclave>;
}

#[derive(Clone, Debug)]
pub struct DefaultHost<E, R> {
    enclave: E,
    _phantom: PhantomData<R>,
}

impl<E, R> DefaultHost<E, R> {
    pub fn new(enclave: E) -> Self {
        Self {
            enclave,
            _phantom: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl<E, R> Host for DefaultHost<E, R>
where
    E: Enclave,
    R: Handler<E>,
{
    type Enclave = E;
    type Request = R;

    async fn enclave_call(
        &self,
        request: Self::Request,
    ) -> HostResult<Self::Request, Self::Enclave> {
        request.handle(&self.enclave).await
    }
}
