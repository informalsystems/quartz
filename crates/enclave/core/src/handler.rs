use crate::{attestor::Attestor, Enclave};

pub type A<E> = <<E as Enclave>::Attestor as Attestor>::Attestation;
pub type RA<E> = <<E as Enclave>::Attestor as Attestor>::RawAttestation;

pub mod instantiate;
pub mod session_create;
pub mod session_set_pubkey;

pub trait Handler<Context> {
    type Error;
    type Response;

    fn handle(&mut self, ctx: &mut Context) -> Result<Self::Response, Self::Error>;
}
