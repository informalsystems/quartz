//! Core types used in the handshake
use std::{
    fmt::{Display, Formatter},
    marker::PhantomData,
    str::FromStr,
};

use hex::FromHexError;
use quartz_contract_core::msg::{
    execute::{
        attested::{Attested, RawAttested},
        session_create::{RawSessionCreate, SessionCreate},
        session_set_pub_key::{RawSessionSetPubKey, SessionSetPubKey},
    },
    instantiate::{CoreInstantiate, RawCoreInstantiate},
    HasDomainType,
};
use quartz_proto::quartz::{
    InstantiateResponse as RawInstantiateResponse,
    SessionCreateResponse as RawSessionCreateResponse,
    SessionSetPubKeyResponse as RawSessionSetPubKeyResponse,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq)]
pub struct InstantiateResponse<A, RA> {
    message: Attested<CoreInstantiate, A>,
    _phantom: PhantomData<RA>,
}

impl<A, RA> InstantiateResponse<A, RA> {
    pub fn new(message: Attested<CoreInstantiate, A>) -> Self {
        Self {
            message,
            _phantom: Default::default(),
        }
    }
    pub fn into_message(self) -> Attested<CoreInstantiate, A> {
        self.message
    }
}

impl<A, RA> From<InstantiateResponse<A, RA>> for RawInstantiateResponse
where
    RA: HasDomainType<DomainType = A> + Serialize,
{
    fn from(value: InstantiateResponse<A, RA>) -> Self {
        let raw_message: RawAttested<RawCoreInstantiate, RA> = value.message.into();
        Self {
            message: serde_json::to_string(&raw_message).expect("infallible serializer"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SessionCreateResponse<A, RA> {
    message: Attested<SessionCreate, A>,
    _phantom: PhantomData<RA>,
}

impl<A, RA> SessionCreateResponse<A, RA> {
    pub fn new(message: Attested<SessionCreate, A>) -> Self {
        Self {
            message,
            _phantom: Default::default(),
        }
    }

    pub fn into_message(self) -> Attested<SessionCreate, A> {
        self.message
    }
}

impl<A, RA> From<SessionCreateResponse<A, RA>> for RawSessionCreateResponse
where
    RA: HasDomainType<DomainType = A> + Serialize,
{
    fn from(value: SessionCreateResponse<A, RA>) -> Self {
        let raw_message: RawAttested<RawSessionCreate, RA> = value.message.into();
        Self {
            message: serde_json::to_string(&raw_message).expect("infallible serializer"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SessionSetPubKeyResponse<A, RA> {
    message: Attested<SessionSetPubKey, A>,
    _phantom: PhantomData<RA>,
}

impl<A, RA> SessionSetPubKeyResponse<A, RA> {
    pub fn new(message: Attested<SessionSetPubKey, A>) -> Self {
        Self {
            message,
            _phantom: Default::default(),
        }
    }

    pub fn into_message(self) -> Attested<SessionSetPubKey, A> {
        self.message
    }
}

impl<A, RA> From<SessionSetPubKeyResponse<A, RA>> for RawSessionSetPubKeyResponse
where
    RA: HasDomainType<DomainType = A> + Serialize,
{
    fn from(value: SessionSetPubKeyResponse<A, RA>) -> Self {
        let raw_message: RawAttested<RawSessionSetPubKey, RA> = value.message.into();
        Self {
            message: serde_json::to_string(&raw_message).expect("infallible serializer"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fmspc(pub [u8; 6]);

impl AsRef<[u8]> for Fmspc {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl FromStr for Fmspc {
    type Err = FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s)?;
        let fmspc: [u8; 6] = bytes
            .try_into()
            .map_err(|_| FromHexError::InvalidStringLength)?;
        Ok(Self(fmspc))
    }
}

impl TryFrom<String> for Fmspc {
    type Error = FromHexError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().parse()
    }
}

impl Display for Fmspc {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self))
    }
}
