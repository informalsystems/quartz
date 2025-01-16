use quartz_common::{
    contract::msg::execute::attested::{HasUserData, RawMsgSansHandler},
    enclave::{attestor::Attestor, handler::Handler, DefaultSharedEnclave, Enclave},
};
use tonic::Status;
use transfers_contract::msg::{AttestedMsg, ExecuteMsg};

use crate::proto::{QueryRequest, UpdateRequest};

pub mod query;
pub mod update;

#[derive(Clone, Debug)]
pub enum EnclaveRequest {
    Update(UpdateRequest),
    Query(QueryRequest),
}

fn attested_msg<T: HasUserData + Clone, A: Attestor>(
    msg: T,
    attestor: A,
) -> Result<AttestedMsg<T, A::RawAttestation>, Status> {
    let attestation = attestor
        .attestation(msg.clone())
        .map_err(|e| Status::internal(e.to_string()))?;

    Ok(AttestedMsg {
        msg: RawMsgSansHandler(msg),
        attestation: attestation.into(),
    })
}

#[async_trait::async_trait]
impl Handler<DefaultSharedEnclave> for EnclaveRequest {
    type Error = Status;
    type Response =
        ExecuteMsg<<<DefaultSharedEnclave as Enclave>::Attestor as Attestor>::RawAttestation>;

    async fn handle(self, ctx: &DefaultSharedEnclave) -> Result<Self::Response, Self::Error> {
        let attestor = ctx.attestor().await;
        match self {
            EnclaveRequest::Update(request) => request
                .handle(ctx)
                .await
                .map(|msg| attested_msg(msg, attestor))?
                .map(ExecuteMsg::Update),
            EnclaveRequest::Query(request) => request
                .handle(ctx)
                .await
                .map(|msg| attested_msg(msg, attestor))?
                .map(ExecuteMsg::QueryResponse),
        }
    }
}
