use cosmwasm_std::{Addr, HexBinary};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContractObligation {
    pub debtor: Addr,
    pub creditor: Addr,
    pub amount: u64,
    #[serde(default)]
    pub salt: HexBinary,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawObligation {
    pub debtor: HexBinary,
    pub creditor: HexBinary,
    pub amount: u64,
    #[serde(default)]
    pub salt: HexBinary,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawEncryptedObligation {
    pub digest: HexBinary,
    pub ciphertext: HexBinary,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmitObligationsMsg {
    pub submit_obligations: SubmitObligationsMsgInner,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmitObligationsMsgInner {
    pub obligations: Vec<RawEncryptedObligation>,
    pub liquidity_sources: Vec<Addr>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RawSetOff {
    SetOff(Vec<HexBinary>),
    Transfer(RawSetOffTransfer),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawSetOffTransfer {
    pub payer: String,
    pub payee: String,
    pub amount: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawOffset {
    pub debtor: HexBinary,
    pub creditor: HexBinary,
    pub amount: u64,
    pub set_off: u64,
}