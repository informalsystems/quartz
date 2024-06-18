use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use cosmwasm_std::{Addr, HexBinary, Uint128};

pub type RawCipherText = HexBinary;

use ecies::{decrypt, encrypt};
use k256::ecdsa::{SigningKey, VerifyingKey};
use quartz_enclave::attestor::Attestor;
use serde::{Deserialize, Serialize};
use tonic::{Request, Response, Result as TonicResult, Status};
use transfers_contracts::{msg::execute::ClearTextTransferRequestMsg, state};

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
    requests: Vec<transfers_contracts::state::Request>,
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
        // Pass in JSON of Requests vector and the STATE
        // Serialize into Requests enum
        let message: RunTransfersRequestMessage = {
            let message = request.into_inner().message;
            serde_json::from_str(&message).map_err(|e| Status::invalid_argument(e.to_string()))?
        };

        // Loop through, decrypt the ciphertexts
        let clear_transfer_requests: Vec<ClearTextTransferRequestMsg> = message
            .requests
            .iter()
            .map(|req| {
                match req {
                    state::Request::Ciphertext(ciphertext) => {
                        let sk = self.sk.lock().unwrap();
                        decrypt_transfer(sk.as_ref().unwrap(), &ciphertext)
                    }
                    _ => unimplemented!(), // what do
                }
            })
            .collect();

        // Decrypt and deserialize
        let mut state = {
            let sk_cpy = self.sk.clone();
            let sk_lock = sk_cpy.as_ref().lock().unwrap();

            decrypt_state(&sk_lock.as_ref().unwrap(), &message.state)
        };

        // Loop through requests and apply onto state
        for tx in clear_transfer_requests {
            // todo: error if balance is less than amount
            state
                .state
                .entry(tx.receiver)
                .and_modify(|curr| *curr += tx.amount)
                .or_insert(tx.amount);
            state
                .state
                .entry(tx.sender)
                .and_modify(|curr| *curr -= tx.amount)
                .or_insert(tx.amount);
        }

        // Encrypt state
        // Gets lock on PrivKey, generates PubKey to encrypt with
        let state_enc = {
            let sk_cpy = self.sk.clone();
            let sk_lock = sk_cpy.as_ref().lock().unwrap();

            let pk = VerifyingKey::from(sk_lock.as_ref().unwrap());

            encrypt_state(RawState::from(state), pk)
        };
        // Create withdraw requests

        // Send to chain

        let message = serde_json::to_string(&RunTransfersResponseMessage {
            ciphertext: state_enc,
            quantity: message.requests.len() as u32,
            withdrawals: BTreeMap::<Addr, Uint128>::default(), // TODO
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

    //TODO: Does this ciphertext need to be wrapped in anything?
    encrypted_state.into()
}
