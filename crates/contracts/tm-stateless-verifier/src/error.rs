use alloc::boxed::Box;

use displaydoc::Display;
use tendermint::{block::Height, Hash};
use tendermint_light_client::{
    builder::error::Error as TmBuilderError, errors::Error as LightClientError,
};

#[derive(Debug, Display)]
pub enum Error {
    /// empty trace
    EmptyTrace,
    /// first block in trace does not match trusted (expected {expected:?}, found {found:?})
    FirstTraceBlockNotTrusted {
        expected: (Height, Hash),
        found: (Height, Hash),
    },
    /// verification failure (`{0}`)
    VerificationFailure(Box<LightClientError>),
    /// failed to build light client (`{0}`)
    LightClientBuildFailure(Box<TmBuilderError>),
}

impl From<LightClientError> for Error {
    fn from(e: LightClientError) -> Self {
        Error::VerificationFailure(Box::new(e))
    }
}

impl From<TmBuilderError> for Error {
    fn from(e: TmBuilderError) -> Self {
        Error::LightClientBuildFailure(Box::new(e))
    }
}
