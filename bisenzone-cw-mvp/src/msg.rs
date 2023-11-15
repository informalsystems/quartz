use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
#[allow(unused)]
use cw20::BalanceResponse;

#[cw_serde]
pub struct InstantiateMsg;

#[cw_serde]
pub enum ExecuteMsg {
    UploadObligation {
        creditor: String,
        amount: Uint128,
        memo: String,
    },
    ApplyCycle {
        path: Vec<String>,
        amount: Uint128,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    #[returns(GetObligationsResponse)]
    GetObligations { creditor: String },
    #[returns(BalanceResponse)]
    Balance { address: String },
}

// We define a custom struct for each query response
#[cw_serde]
pub struct GetObligationsResponse {
    pub obligations: Vec<(String, Uint128)>,
}
