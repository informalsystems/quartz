use cosmwasm_std::{HexBinary, StdError};
use quartz_cw::state::{Config, RawConfig};
use quartz_proto::quartz::InstantiateResponse as RawInstantiateResponse;
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
