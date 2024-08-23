use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub root_cert: String,
}

#[cw_serde]
pub struct ExecuteMsg {
    pub tcb_info: String,
    pub certificate: String,
    pub time: Option<String>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetTcbInfoResponse)]
    GetTcbInfo { fmspc: String },
}

#[cw_serde]
pub struct GetTcbInfoResponse {
    pub tcb_info: serde_json::Value,
}
