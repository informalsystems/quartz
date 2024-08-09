use serde::Serialize;

use crate::response::Response;

#[derive(Clone, Debug, Serialize, Default)]
pub struct ContractTxResponse {
    pub tx_hash: String,
}

impl From<ContractTxResponse> for Response {
    fn from(response: ContractTxResponse) -> Self {
        Self::ContractTx(response)
    }
}
