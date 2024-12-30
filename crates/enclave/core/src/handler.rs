use tonic::{Request, Response};

use crate::{attestor::Attestor, Enclave};

pub type A<E> = <<E as Enclave>::Attestor as Attestor>::Attestation;
pub type RA<E> = <<E as Enclave>::Attestor as Attestor>::RawAttestation;

pub mod instantiate;
pub mod session_create;
pub mod session_set_pubkey;

#[async_trait::async_trait]
pub trait Handler<Context>: Send + Sync + 'static {
    type Error;
    type Response;

    async fn handle(self, ctx: &Context) -> Result<Self::Response, Self::Error>;
}

#[async_trait::async_trait]
impl<T, C> Handler<C> for Request<T>
where
    T: Handler<C>,
    C: Send + Sync,
{
    type Error = T::Error;
    type Response = Response<T::Response>;

    async fn handle(self, ctx: &C) -> Result<Self::Response, Self::Error> {
        let request = self.into_inner();
        let response = request.handle(ctx).await?;
        Ok(Response::new(response.into()))
    }
}
