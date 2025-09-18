use std::vec::IntoIter;

use cosmwasm_std::HexBinary;
use ecies::{decrypt, encrypt};
use k256::ecdsa::{SigningKey, VerifyingKey};
use quartz_common::{
    contract::msg::execute::attested::{HasUserData, RawNoop},
    enclave::{
        attestor::{Attestor, DefaultAttestor},
        handler::Handler,
        Enclave,
    },
};
use tonic::Status;
use transfers_contract::msg::{execute::ClearTextTransferRequestMsg, AttestedMsg, ExecuteMsg};

use crate::{
    proto::{QueryRequest, UpdateRequest},
    state::{AppEnclave, Balance, State},
};

pub mod query;
pub mod update;

pub type EnclaveMsg = ExecuteMsg<<DefaultAttestor as Attestor>::RawAttestation>;
pub type EnclaveResponse = IntoIter<EnclaveMsg>;

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
        msg: RawNoop(msg),
        attestation: attestation.into(),
    })
}

#[async_trait::async_trait]
impl Handler<AppEnclave> for EnclaveRequest {
    type Error = Status;
    type Response = EnclaveResponse;

    async fn handle(self, ctx: &AppEnclave) -> Result<Self::Response, Self::Error> {
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
        .map(|msg| vec![msg].into_iter())
    }
}

fn decrypt_transfer(
    sk: &SigningKey,
    ciphertext: &HexBinary,
) -> Result<ClearTextTransferRequestMsg, Status> {
    let o =
        decrypt(&sk.to_bytes(), ciphertext).map_err(|e| Status::invalid_argument(e.to_string()))?;

    serde_json::from_slice(&o)
        .map_err(|e| Status::internal(format!("Could not deserialize transfer {}", e)))
}

fn decrypt_state(sk: &SigningKey, ciphertext: &[u8]) -> Result<State, Status> {
    let o =
        decrypt(&sk.to_bytes(), ciphertext).map_err(|e| Status::invalid_argument(e.to_string()))?;
    serde_json::from_slice(&o).map_err(|e| Status::invalid_argument(e.to_string()))
}

fn encrypt_state(state: State, enclave_pk: VerifyingKey) -> Result<HexBinary, Status> {
    let serialized_state = serde_json::to_string(&state).expect("infallible serializer");

    match encrypt(&enclave_pk.to_sec1_bytes(), serialized_state.as_bytes()) {
        Ok(encrypted_state) => Ok(encrypted_state.into()),
        Err(e) => Err(Status::internal(format!("Encryption error: {}", e))),
    }
}

fn encrypt_balance(balance: Balance, ephemeral_pk: VerifyingKey) -> Result<HexBinary, Status> {
    let serialized_balance = serde_json::to_string(&balance).expect("infallible serializer");

    match encrypt(&ephemeral_pk.to_sec1_bytes(), serialized_balance.as_bytes()) {
        Ok(encrypted_balance) => Ok(encrypted_balance.into()),
        Err(e) => Err(Status::internal(format!("Encryption error: {}", e))),
    }
}
