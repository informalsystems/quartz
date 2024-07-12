use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg<'a> {
    pub root: &'a str,
}

#[cw_serde]
pub struct ExecuteMsg<'a> {
    pub fmspc: [u8; 6],
    pub tcb_info: &'a str,
    pub certificate: &'a str,
    pub time: u64
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    #[returns(GetCountResponse)]
    GetCount {},
}

// We define a custom struct for each query response
#[cw_serde]
pub struct GetCountResponse {
    pub count: i32,
}
