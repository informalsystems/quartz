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
    pub time: &'a str,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg<'a> {
    // GetCount returns the current count as a json-encoded number
    #[returns(GetTcbInfoResponse)]
    GetTcbInfo { fmspc: [u8; 6], time: &'a str },
}

#[cw_serde]
pub struct GetTcbInfoResponse {
    pub tcb_info: String,
}
