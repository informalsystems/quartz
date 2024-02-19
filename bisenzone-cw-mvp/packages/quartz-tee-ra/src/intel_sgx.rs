use thiserror::Error;

pub mod epid;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Specified user data does not match the report")]
    UserDataMismatch,
    #[error("Specified MRENCLAVE does not match the report")]
    MrEnclaveMismatch,
    #[error("EPID specific error: {0}")]
    Epid(#[from] epid::Error),
}
