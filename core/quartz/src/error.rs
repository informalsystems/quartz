use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum QuartzError {
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tendermint_rpc::Error),
    #[error("Tonic transport error: {0}")]
    TonicTransport(#[from] tonic::transport::Error),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Other error: {0}")]
    Other(String),
}
