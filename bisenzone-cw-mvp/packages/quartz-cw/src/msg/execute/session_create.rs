use cosmwasm_schema::cw_serde;
use cosmwasm_std::{HexBinary, StdError};

use crate::msg::execute::attested::HasUserData;
use crate::msg::HasDomainType;
use crate::state::{Nonce, UserData};

#[derive(Clone, Debug, PartialEq)]
pub struct SessionCreate {
    nonce: Nonce,
}

impl SessionCreate {
    pub fn new(nonce: Nonce) -> Self {
        Self { nonce }
    }

    pub fn into_nonce(self) -> Nonce {
        self.nonce
    }
}

#[cw_serde]
pub struct RawSessionCreate {
    nonce: HexBinary,
}

impl TryFrom<RawSessionCreate> for SessionCreate {
    type Error = StdError;

    fn try_from(value: RawSessionCreate) -> Result<Self, Self::Error> {
        let nonce = value.nonce.to_array()?;
        Ok(Self { nonce })
    }
}

impl From<SessionCreate> for RawSessionCreate {
    fn from(value: SessionCreate) -> Self {
        Self {
            nonce: value.nonce.into(),
        }
    }
}

impl HasDomainType for RawSessionCreate {
    type DomainType = SessionCreate;
}

impl HasUserData for SessionCreate {
    fn user_data(&self) -> UserData {
        let mut user_data = [0u8; 64];
        user_data[0..32].copy_from_slice(&self.nonce);
        user_data
    }
}
