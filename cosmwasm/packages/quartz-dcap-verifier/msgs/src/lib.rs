use cosmwasm_schema::{cw_serde, QueryResponses};

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
        quote: Vec<u8>,
        collateral: Vec<u8>,
        identities: Vec<u8>,
    },
}
