use async_trait::async_trait;

use crate::types::{ObligatoObligation, ObligatoSetOff};

pub mod http;
pub mod mock;

#[async_trait]
pub trait Client {
    type Error;

    async fn get_obligations(&self) -> Result<Vec<ObligatoObligation>, Self::Error>;

    async fn set_setoffs(&self, setoffs: Vec<ObligatoSetOff>) -> Result<(), Self::Error>;
}
