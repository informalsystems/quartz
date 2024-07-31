use bip32::secp256k1::ecdsa::VerifyingKey;
use cosmwasm_std::{Addr, HexBinary};
use cw_tee_mtcs::state::LiquiditySource;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Eq, Hash)]
pub struct ObligatoObligation {
    pub id: Uuid,
    pub debtor_id: Uuid,
    pub creditor_id: Uuid,
    pub amount: u64,
}

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

#[derive(Clone, Debug)]
pub struct Obligation {
    pub debtor: VerifyingKey,
    pub creditor: VerifyingKey,
    pub amount: u64,
    pub salt: [u8; 64],
}

impl From<Obligation> for RawObligation {
    fn from(obligation: Obligation) -> Self {
        Self {
            debtor: obligation.debtor.to_sec1_bytes().into_vec().into(),
            creditor: obligation.creditor.to_sec1_bytes().into_vec().into(),
            amount: obligation.amount,
            salt: obligation.salt.into(),
        }
    }
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
    pub liquidity_sources: Vec<LiquiditySource>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmitObligatioMsg {
    pub submit_obligations: SubmitObligatoObligationsMsgInner,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmitObligatoObligationsMsgInner {
    pub obligations: Vec<RawEncryptedObligation>,
    pub liquidity_sources: Vec<HexBinary>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ObligatoSetOff {
    pub debtor_id: Uuid,
    pub creditor_id: Uuid,
    pub amount: u64,
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
