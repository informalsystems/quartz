use cosmwasm_schema::cw_serde;
use cosmwasm_std::HexBinary;
use cw_storage_plus::Item;
use k256::ecdsa::VerifyingKey;

pub type MrEnclave = [u8; 32];
pub type Nonce = [u8; 32];
pub type UserData = [u8; 64];

#[cw_serde]
pub struct Config {
    mr_enclave: HexBinary,
}

impl Config {
    pub fn new(mr_enclave: MrEnclave) -> Self {
        Self {
            mr_enclave: mr_enclave.into(),
        }
    }

    pub fn mr_enclave(&self) -> &HexBinary {
        &self.mr_enclave
    }
}

#[cw_serde]
pub struct Session {
    nonce: HexBinary,
    pub_key: Option<HexBinary>,
}

impl Session {
    pub fn create(nonce: Nonce) -> Self {
        Self {
            nonce: nonce.into(),
            pub_key: None,
        }
    }

    pub fn with_pub_key(mut self, nonce: Nonce, pub_key: VerifyingKey) -> Option<Self> {
        if self.nonce == nonce && self.pub_key.is_none() {
            self.pub_key = Some(pub_key.to_sec1_bytes().into_vec().into());
            Some(self)
        } else {
            None
        }
    }
}

pub const CONFIG: Item<'_, Config> = Item::new("quartz_config");
pub const SESSION: Item<'_, Session> = Item::new("quartz_session");
