use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::HexBinary;

#[cw_serde]
pub struct InstantiateMsg;

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Verify a DCAP attestation
    #[returns(())]
    VerifyDcapAttestation {
        quote: HexBinary,
        collateral: HexBinary,
        identities: Option<Vec<u8>>,
    },
}
