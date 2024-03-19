use cosmwasm_schema::cw_serde;
use cosmwasm_std::{HexBinary, StdError};
use k256::ecdsa::VerifyingKey;
use sha2::{Digest, Sha256};

use crate::error::Error;
use crate::msg::execute::attested::HasUserData;
use crate::msg::HasDomainType;
use crate::state::{Nonce, UserData};

#[derive(Clone, Debug, PartialEq)]
pub struct SessionSetPubKey {
    nonce: Nonce,
    pub_key: VerifyingKey,
}

impl SessionSetPubKey {
    pub fn new(nonce: Nonce, pub_key: VerifyingKey) -> Self {
        Self { nonce, pub_key }
    }

    pub fn into_tuple(self) -> (Nonce, VerifyingKey) {
        (self.nonce, self.pub_key)
    }
}

#[cw_serde]
pub struct RawSessionSetPubKey {
    nonce: HexBinary,
    pub_key: HexBinary,
}

impl TryFrom<RawSessionSetPubKey> for SessionSetPubKey {
    type Error = StdError;

    fn try_from(value: RawSessionSetPubKey) -> Result<Self, Self::Error> {
        let nonce = value.nonce.to_array()?;
        let pub_key = VerifyingKey::from_sec1_bytes(&value.pub_key)
            .map_err(Error::from)
            .map_err(|e| StdError::generic_err(e.to_string()))?;
        Ok(Self { nonce, pub_key })
    }
}

impl From<SessionSetPubKey> for RawSessionSetPubKey {
    fn from(value: SessionSetPubKey) -> Self {
        Self {
            nonce: value.nonce.into(),
            pub_key: value.pub_key.to_sec1_bytes().into_vec().into(),
        }
    }
}

impl HasDomainType for RawSessionSetPubKey {
    type DomainType = SessionSetPubKey;
}

impl HasUserData for SessionSetPubKey {
    fn user_data(&self) -> UserData {
        let mut hasher = Sha256::new();
        hasher.update(self.nonce);
        hasher.update(self.pub_key.to_sec1_bytes());
        let digest: [u8; 32] = hasher.finalize().into();

        let mut user_data = [0u8; 64];
        user_data[0..32].copy_from_slice(&digest);
        user_data
    }
}
