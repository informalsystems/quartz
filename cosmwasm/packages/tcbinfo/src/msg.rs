use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub root: String,
}

#[cw_serde]
pub struct ExecuteMsg {
    pub tcb_info: String,
    pub certificate: String,
    pub time: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetTcbInfoResponse)]
    GetTcbInfo { fmspc: [u8; 6], time: String },
}

#[cw_serde]
pub struct GetTcbInfoResponse {
    pub tcb_info: String,
}
