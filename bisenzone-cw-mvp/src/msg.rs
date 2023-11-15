use std::collections::HashMap;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg;

#[cw_serde]
pub enum ExecuteMsg {
    UploadObligation {
        creditor: Addr,
        amount: u64,
        memo: String,
    },
    ApplyCycle {
        path: Vec<Addr>,
        amount: u64,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    #[returns(GetObligationsResponse)]
    GetObligations { creditor: Addr },
}

// We define a custom struct for each query response
#[cw_serde]
pub struct GetObligationsResponse {
    pub obligations: HashMap<Addr, u64>,
}
