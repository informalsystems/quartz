use std::collections::btree_map::Entry;

use cosmrs::AccountId;
use cosmwasm_std::{Addr, HexBinary, Uint128};
use ecies::{decrypt, encrypt};
use k256::ecdsa::{SigningKey, VerifyingKey};
use quartz_common::enclave::{
    handler::Handler,
    key_manager::KeyManager,
    kv_store::{ConfigKey, ConfigKeyName, ContractKey, ContractKeyName, KvStore},
    server::ProofOfPublication,
    Enclave,
};
use tonic::Status;
use transfers_contract::{
    msg::{
        execute,
        execute::{ClearTextTransferRequestMsg, Request as TransfersRequest},
    },
    state::REQUESTS_KEY,
};

use crate::{
    proto::UpdateRequest,
    state::State,
    transfers_server::{RawCipherText, UpdateRequestMessage},
};

#[async_trait::async_trait]
impl<E: Enclave> Handler<E> for UpdateRequest
where
    E: Enclave<Contract = AccountId>,
    E::KeyManager: KeyManager<PubKey = VerifyingKey, PrivKey = SigningKey>,
{
    type Error = Status;
    type Response = execute::UpdateMsg;

    async fn handle(self, ctx: &E) -> Result<Self::Response, Self::Error> {
        // verify proof
        let proof: ProofOfPublication<UpdateRequestMessage> = {
            let message = self.message;
            serde_json::from_str(&message).map_err(|e| Status::invalid_argument(e.to_string()))?
        };
        let contract = ctx
            .store()
            .await
            .get(ContractKey::new(ContractKeyName))
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("contract not found"))?;
        let config = ctx
            .store()
            .await
            .get(ConfigKey::new(ConfigKeyName))
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("config not found"))?;
        let (proof_value, message) = proof
            .verify(
                config.light_client_opts(),
                contract,
                REQUESTS_KEY.to_string(),
                None,
            )
            .map_err(Status::failed_precondition)?;

        let proof_value_matches_msg =
            serde_json::to_string(&message.requests).is_ok_and(|s| s.as_bytes() == proof_value);
        if !proof_value_matches_msg {
            return Err(Status::failed_precondition("proof verification"));
        }

        // Decrypt and deserialize the state
        let mut state = match &message.state.to_vec()[..] {
            &[0] => State::default(),
            state_bytes => {
                let sk = ctx
                    .key_manager()
                    .await
                    .priv_key()
                    .await
                    .ok_or_else(|| Status::internal("failed to get private key"))?;
                decrypt_state(&sk, state_bytes)?
            }
        };

        let requests_len = message.requests.len() as u32;

        // Instantiate empty withdrawals map to include in response (Update message to smart contract)
        let mut withdrawals_response: Vec<(Addr, Uint128)> = Vec::<(Addr, Uint128)>::new();

        // let pending_sequenced_requests = message
        //     .requests
        //     .iter()
        //     .filter(|req| matches!(req, TransfersRequest::Transfer(_)))
        //     .count();

        // Loop through requests, match on cases, and apply changes to state
        for req in message.requests {
            match req {
                TransfersRequest::Transfer(ciphertext) => {
                    // TODO: ensure_seq_num_consistency(message.seq_num, pending_sequenced_requests)?;

                    // Decrypt transfer ciphertext into cleartext struct (acquires lock on enclave sk to do so)
                    let transfer: ClearTextTransferRequestMsg = {
                        let sk = ctx
                            .key_manager()
                            .await
                            .priv_key()
                            .await
                            .ok_or_else(|| Status::internal("failed to get private key"))?;

                        decrypt_transfer(&sk, &ciphertext)?
                    };
                    if let Entry::Occupied(mut entry) = state.state.entry(transfer.sender) {
                        let balance = entry.get();
                        if balance >= &transfer.amount {
                            entry.insert(balance - transfer.amount);

                            state
                                .state
                                .entry(transfer.receiver)
                                .and_modify(|bal| *bal += transfer.amount)
                                .or_insert(transfer.amount);
                        }
                        // TODO: handle errors
                    }
                }
                TransfersRequest::Withdraw(receiver) => {
                    // If a user with no balance requests withdraw, withdraw request for 0 coins gets processed
                    // TODO: A no-op seems like a bad design choice in a privacy system
                    if let Some(withdraw_bal) = state.state.remove(&receiver) {
                        withdrawals_response.push((receiver, withdraw_bal));
                    }
                }
                TransfersRequest::Deposit(sender, amount) => {
                    state
                        .state
                        .entry(sender)
                        .and_modify(|bal| *bal += amount)
                        .or_insert(amount);
                }
            }
        }

        // Encrypt state
        // Gets lock on PrivKey, generates PubKey to encrypt with
        let state_enc = {
            let pk = ctx
                .key_manager()
                .await
                .pub_key()
                .await
                .ok_or_else(|| Status::internal("failed to get public key"))?;

            encrypt_state(state, pk).map_err(|e| Status::invalid_argument(e.to_string()))?
        };

        // Prepare message to chain
        let msg = execute::UpdateMsg {
            ciphertext: state_enc,
            quantity: requests_len,
            withdrawals: withdrawals_response,
        };

        Ok(msg)
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

fn encrypt_state(state: State, enclave_pk: VerifyingKey) -> Result<RawCipherText, Status> {
    let serialized_state = serde_json::to_string(&state).expect("infallible serializer");

    match encrypt(&enclave_pk.to_sec1_bytes(), serialized_state.as_bytes()) {
        Ok(encrypted_state) => Ok(encrypted_state.into()),
        Err(e) => Err(Status::internal(format!("Encryption error: {}", e))),
    }
}

// fn ensure_seq_num_consistency(
//     seq_num_in_store: &mut u64,
//     seq_num_on_chain: u64,
//     pending_sequenced_requests: usize,
// ) -> Result<(), Status> {
//     if seq_num_on_chain < *seq_num_in_store {
//         return Err(Status::failed_precondition("replay attempted"));
//     }
//
//     // make sure number of pending requests are equal to the diff b/w on-chain v/s in-mem seq num
//     let seq_num_diff = seq_num_on_chain - *seq_num_in_store;
//     if seq_num_diff != pending_sequenced_requests as u64 {
//         return Err(Status::failed_precondition(format!(
//             "seq_num_diff mismatch: num({seq_num_diff}) v/s diff({pending_sequenced_requests})"
//         )));
//     }
//
//     *seq_num_in_store = seq_num_on_chain;
//
//     Ok(())
// }
