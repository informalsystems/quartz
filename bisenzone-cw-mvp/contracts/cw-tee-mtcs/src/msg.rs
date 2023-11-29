use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg;

#[cw_serde]
pub enum ExecuteMsg {}

pub mod execute {
    use super::*;

    #[cw_serde]
    pub struct Nonce([u8; 32]);

    #[cw_serde]
    pub struct JoinComputeNodeMsg {
        compute_node_pub_key: String,
        nonce: Nonce,
    }

    #[cw_serde]
    pub struct ShareEpochKeyMsg {
        compute_node_pub_key: String,
        nonce: Nonce,
    }
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
