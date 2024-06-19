use std::{
    collections::{btree_map::Entry, BTreeMap},
    sync::{Arc, Mutex},
};

use cosmwasm_std::{Addr, HexBinary, Uint128};

pub type RawCipherText = HexBinary;

use ecies::{decrypt, encrypt};
use k256::ecdsa::{SigningKey, VerifyingKey};
use quartz_enclave::attestor::Attestor;
use serde::{Deserialize, Serialize};
use tonic::{Request, Response, Result as TonicResult, Status};
use transfers_contracts::msg::execute::{ClearTextTransferRequestMsg, Request as TransfersRequest};

use crate::{
    proto::{settlement_server::Settlement, RunTransfersRequest, RunTransfersResponse},
    state::{RawState, State},
};

#[derive(Clone, Debug)]
pub struct TransfersService<A> {
    sk: Arc<Mutex<Option<SigningKey>>>,
    _attestor: A,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunTransfersRequestMessage {
    state: HexBinary,
    requests: Vec<TransfersRequest>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunTransfersResponseMessage {
    ciphertext: HexBinary,
    quantity: u32,
    withdrawals: BTreeMap<Addr, Uint128>,
}

impl<A> TransfersService<A>
where
    A: Attestor,
{
    pub fn new(sk: Arc<Mutex<Option<SigningKey>>>, _attestor: A) -> Self {
        Self { sk, _attestor }
    }
}

#[tonic::async_trait]
impl<A> Settlement for TransfersService<A>
where
    A: Attestor + Send + Sync + 'static,
{
    async fn run(
        &self,
        request: Request<RunTransfersRequest>,
    ) -> TonicResult<Response<RunTransfersResponse>> {
        // Request contains a serialized json string

        // Serialize request into struct containing State and the Requests vec
        let message: RunTransfersRequestMessage = {
            let message = request.into_inner().message;
            serde_json::from_str(&message).map_err(|e| Status::invalid_argument(e.to_string()))?
        };

        // Decrypt and deserialize the state
        let mut state = {
            let sk_cpy = self.sk.clone();
            let sk_lock = sk_cpy.as_ref().lock().unwrap();

            decrypt_state(&sk_lock.as_ref().unwrap(), &message.state)
        };

        let requests_len = message.requests.len() as u32;
        // Instantiate empty withdrawals map to include in response (Update message to smart contract)
        let mut withdrawals_response = BTreeMap::<Addr, Uint128>::new();

        // Loop through requests, match on cases, and apply changes to state
        for req in message.requests {
            match req {
                TransfersRequest::Transfer(ciphertext) => {
                    // Decrypt transfer ciphertext into cleartext struct (acquires lock on enclave sk to do so)
                    let transfer: ClearTextTransferRequestMsg = {
                        let sk = self.sk.lock().unwrap();

                        decrypt_transfer(sk.as_ref().unwrap(), &ciphertext)
                    };
                    if let Entry::Occupied(mut entry) = state.state.entry(transfer.sender) {
                        let balance = entry.get();
                        if balance >= &transfer.amount {
                            entry.insert(balance - transfer.amount);
                        }
                        // TODO: handle errors
                    }

                    state
                        .state
                        .entry(transfer.receiver)
                        .and_modify(|bal| *bal += transfer.amount)
                        .or_insert(transfer.amount);
                }
                TransfersRequest::Withdraw(receiver, amount) => {
                    if let Entry::Occupied(mut entry) = state.state.entry(receiver.clone()) {
                        let balance = entry.get();
                        if balance >= &amount {
                            entry.insert(balance - amount);
                        }
                        // TODO: handle errors
                    }

                    withdrawals_response.insert(receiver, amount);
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
            let sk_cpy = self.sk.clone();
            let sk_lock = sk_cpy.as_ref().lock().unwrap();

            let pk = VerifyingKey::from(sk_lock.as_ref().unwrap());

            encrypt_state(RawState::from(state), pk)
        };

        // Send to chain
        let message = serde_json::to_string(&RunTransfersResponseMessage {
            ciphertext: state_enc,
            quantity: requests_len,
            withdrawals: withdrawals_response,
        })
        .unwrap();

        Ok(Response::new(RunTransfersResponse { message }))
    }
}

fn decrypt_transfer(sk: &SigningKey, ciphertext: &HexBinary) -> ClearTextTransferRequestMsg {
    let o = decrypt(&sk.to_bytes(), ciphertext).unwrap();

    serde_json::from_slice(&o).unwrap()
}

//TODO: consider using generics for these decrypt functions
fn decrypt_state(sk: &SigningKey, ciphertext: &HexBinary) -> State {
    let o: RawState = {
        let o = decrypt(&sk.to_bytes(), ciphertext).unwrap();
        serde_json::from_slice(&o).unwrap()
    };

    State::try_from(o).unwrap()
}

fn encrypt_state(state: RawState, enclave_pk: VerifyingKey) -> RawCipherText {
    let serialized_state = serde_json::to_string(&state).expect("infallible serializer");
    let encrypted_state =
        encrypt(&enclave_pk.to_sec1_bytes(), serialized_state.as_bytes()).unwrap();

    encrypted_state.into()
}
