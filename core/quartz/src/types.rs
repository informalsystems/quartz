use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use cosmwasm_std::{HexBinary, StdError};
use hex::FromHexError;
use k256::ecdsa::VerifyingKey;
use quartz_cw::{
    error::Error as QuartzCwError,
    state::{Config, Nonce, RawConfig},
};
use quartz_proto::quartz::{
    InstantiateResponse as RawInstantiateResponse,
    SessionCreateResponse as RawSessionCreateResponse,
    SessionSetPubKeyResponse as RawSessionSetPubKeyResponse,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq)]
pub struct InstantiateResponse {
    message: InstantiateResponseMsg,
}

impl InstantiateResponse {
    pub fn new(config: Config, quote: Vec<u8>) -> Self {
        Self {
            message: InstantiateResponseMsg { config, quote },
        }
    }

    pub fn quote(&self) -> &[u8] {
        &self.message.quote
    }

    pub fn into_message(self) -> InstantiateResponseMsg {
        self.message
    }
}

impl TryFrom<RawInstantiateResponse> for InstantiateResponse {
    type Error = StdError;

    fn try_from(value: RawInstantiateResponse) -> Result<Self, Self::Error> {
        let raw_message: RawInstantiateResponseMsg = serde_json::from_str(&value.message)
            .map_err(|e| StdError::parse_err("RawInstantiateResponseMsg", e))?;
        Ok(Self {
            message: raw_message.try_into()?,
        })
    }
}

impl From<InstantiateResponse> for RawInstantiateResponse {
    fn from(value: InstantiateResponse) -> Self {
        let raw_message: RawInstantiateResponseMsg = value.message.into();
        Self {
            message: serde_json::to_string(&raw_message).expect("infallible serializer"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InstantiateResponseMsg {
    config: Config,
    quote: Vec<u8>,
}

impl InstantiateResponseMsg {
    pub fn into_tuple(self) -> (Config, Vec<u8>) {
        (self.config, self.quote)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RawInstantiateResponseMsg {
    config: RawConfig,
    quote: HexBinary,
}

impl TryFrom<RawInstantiateResponseMsg> for InstantiateResponseMsg {
    type Error = StdError;

    fn try_from(value: RawInstantiateResponseMsg) -> Result<Self, Self::Error> {
        Ok(Self {
            config: value.config.try_into()?,
            quote: value.quote.into(),
        })
    }
}

impl From<InstantiateResponseMsg> for RawInstantiateResponseMsg {
    fn from(value: InstantiateResponseMsg) -> Self {
        Self {
            config: value.config.into(),
            quote: value.quote.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SessionCreateResponse {
    message: SessionCreateResponseMsg,
}

impl SessionCreateResponse {
    pub fn new(nonce: Nonce, quote: Vec<u8>) -> Self {
        Self {
            message: SessionCreateResponseMsg { nonce, quote },
        }
    }

    pub fn quote(&self) -> &[u8] {
        &self.message.quote
    }

    pub fn into_message(self) -> SessionCreateResponseMsg {
        self.message
    }
}

impl TryFrom<RawSessionCreateResponse> for SessionCreateResponse {
    type Error = StdError;

    fn try_from(value: RawSessionCreateResponse) -> Result<Self, Self::Error> {
        let raw_message: RawSessionCreateResponseMsg = serde_json::from_str(&value.message)
            .map_err(|e| StdError::parse_err("RawSessionCreateResponseMsg", e))?;
        Ok(Self {
            message: raw_message.try_into()?,
        })
    }
}

impl From<SessionCreateResponse> for RawSessionCreateResponse {
    fn from(value: SessionCreateResponse) -> Self {
        let raw_message: RawSessionCreateResponseMsg = value.message.into();
        Self {
            message: serde_json::to_string(&raw_message).expect("infallible serializer"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SessionCreateResponseMsg {
    nonce: Nonce,
    quote: Vec<u8>,
}

impl SessionCreateResponseMsg {
    pub fn into_tuple(self) -> (Nonce, Vec<u8>) {
        (self.nonce, self.quote)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RawSessionCreateResponseMsg {
    nonce: HexBinary,
    quote: HexBinary,
}

impl TryFrom<RawSessionCreateResponseMsg> for SessionCreateResponseMsg {
    type Error = StdError;

    fn try_from(value: RawSessionCreateResponseMsg) -> Result<Self, Self::Error> {
        Ok(Self {
            nonce: value.nonce.to_array()?,
            quote: value.quote.into(),
        })
    }
}

impl From<SessionCreateResponseMsg> for RawSessionCreateResponseMsg {
    fn from(value: SessionCreateResponseMsg) -> Self {
        Self {
            nonce: value.nonce.into(),
            quote: value.quote.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SessionSetPubKeyResponse {
    message: SessionSetPubKeyResponseMsg,
}

impl SessionSetPubKeyResponse {
    pub fn new(nonce: Nonce, pub_key: VerifyingKey, quote: Vec<u8>) -> Self {
        Self {
            message: SessionSetPubKeyResponseMsg {
                nonce,
                pub_key,
                quote,
            },
        }
    }

    pub fn quote(&self) -> &[u8] {
        &self.message.quote
    }

    pub fn into_message(self) -> SessionSetPubKeyResponseMsg {
        self.message
    }
}

impl TryFrom<RawSessionSetPubKeyResponse> for SessionSetPubKeyResponse {
    type Error = StdError;

    fn try_from(value: RawSessionSetPubKeyResponse) -> Result<Self, Self::Error> {
        let raw_message: RawSessionSetPubKeyResponseMsg = serde_json::from_str(&value.message)
            .map_err(|e| StdError::parse_err("RawSessionSetPubKeyResponseMsg", e))?;
        Ok(Self {
            message: raw_message.try_into()?,
        })
    }
}

impl From<SessionSetPubKeyResponse> for RawSessionSetPubKeyResponse {
    fn from(value: SessionSetPubKeyResponse) -> Self {
        let raw_message: RawSessionSetPubKeyResponseMsg = value.message.into();
        Self {
            message: serde_json::to_string(&raw_message).expect("infallible serializer"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SessionSetPubKeyResponseMsg {
    nonce: Nonce,
    pub_key: VerifyingKey,
    quote: Vec<u8>,
}

impl SessionSetPubKeyResponseMsg {
    pub fn into_tuple(self) -> (VerifyingKey, Vec<u8>) {
        (self.pub_key, self.quote)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RawSessionSetPubKeyResponseMsg {
    nonce: HexBinary,
    pub_key: HexBinary,
    quote: HexBinary,
}

impl TryFrom<RawSessionSetPubKeyResponseMsg> for SessionSetPubKeyResponseMsg {
    type Error = StdError;

    fn try_from(value: RawSessionSetPubKeyResponseMsg) -> Result<Self, Self::Error> {
        let pub_key = VerifyingKey::from_sec1_bytes(&value.pub_key)
            .map_err(QuartzCwError::from)
            .map_err(|e| StdError::generic_err(e.to_string()))?;
        Ok(Self {
            nonce: value.nonce.to_array()?,
            pub_key,
            quote: value.quote.into(),
        })
    }
}

impl From<SessionSetPubKeyResponseMsg> for RawSessionSetPubKeyResponseMsg {
    fn from(value: SessionSetPubKeyResponseMsg) -> Self {
        Self {
            nonce: value.nonce.into(),
            pub_key: value.pub_key.to_sec1_bytes().into_vec().into(),
            quote: value.quote.into(),
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
