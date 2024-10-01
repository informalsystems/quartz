use std::collections::BTreeMap;

use cosmwasm_std::{Addr, HexBinary, Uint128};
use ecies::encrypt;
use k256::ecdsa::VerifyingKey;
use serde::{Deserialize, Serialize};
use transfers_contract::msg::execute::ClearTextTransferRequestMsg;

pub type RawCipherText = HexBinary;

#[derive(Clone, Debug)]
pub struct State {
    pub state: BTreeMap<Addr, Uint128>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct RawState {
    pub state: BTreeMap<Addr, Uint128>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawEncryptedState {
    pub ciphertext: HexBinary,
}

impl From<State> for RawState {
    fn from(o: State) -> Self {
        Self { state: o.state }
    }
}

impl TryFrom<RawState> for State {
    type Error = anyhow::Error;

    fn try_from(o: RawState) -> Result<Self, anyhow::Error> {
        Ok(Self { state: o.state })
    }
}

fn main() {
    let msg = ClearTextTransferRequestMsg {
        sender: Addr::unchecked("alice"),
        receiver: Addr::unchecked("bob"),
        amount: Uint128::from(100_u32),
    };

    let decoded: Vec<u8> =
        hex::decode("03e8d63b96a3b3fa442f0a8f39a580f5e898dab7b86eaa685466e82d79022eedff")
            .expect("Decoding failed");
    let sk = VerifyingKey::from_sec1_bytes(&decoded).unwrap();

    let serialized_state = serde_json::to_string(&msg).expect("infallible serializer");
    let encrypted_state = encrypt(&sk.to_sec1_bytes(), serialized_state.as_bytes()).unwrap();

    let result: HexBinary = encrypted_state.into();

    println!("{}", result);
}
