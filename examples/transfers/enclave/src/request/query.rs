use cosmwasm_std::{Addr, HexBinary, Uint128};
use ecies::{decrypt, encrypt};
use k256::ecdsa::{SigningKey, VerifyingKey};
use quartz_common::enclave::{
    handler::Handler, key_manager::KeyManager, DefaultSharedEnclave, Enclave,
};
use serde::{Deserialize, Serialize};
use tonic::Status;
use transfers_contract::msg::execute;

use crate::{
    proto::QueryRequest,
    state::{Balance, State},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryRequestMessage {
    pub state: HexBinary,
    pub address: Addr,
    pub ephemeral_pubkey: HexBinary,
}

#[async_trait::async_trait]
impl Handler<DefaultSharedEnclave<()>> for QueryRequest {
    type Error = Status;
    type Response = execute::QueryResponseMsg;

    async fn handle(self, ctx: &DefaultSharedEnclave<()>) -> Result<Self::Response, Self::Error> {
        let message: QueryRequestMessage = {
            let message: String = self.message;
            serde_json::from_str(&message).map_err(|e| Status::invalid_argument(e.to_string()))?
        };

        // Decrypt and deserialize the state
        let state = match &message.state.to_vec()[..] {
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

        let bal = match state.state.get(&message.address) {
            Some(balance) => Balance { balance: *balance },
            None => Balance {
                balance: Uint128::new(0),
            },
        };

        // Parse the ephemeral public key
        let ephemeral_pubkey =
            VerifyingKey::from_sec1_bytes(&message.ephemeral_pubkey).map_err(|e| {
                Status::invalid_argument(format!("Invalid ephemeral public key: {}", e))
            })?;

        // Encrypt the balance using the ephemeral public key
        let bal_enc = encrypt_balance(bal, ephemeral_pubkey)
            .map_err(|e| Status::internal(format!("Encryption error: {}", e)))?;

        // Prepare message to chain
        let msg = execute::QueryResponseMsg {
            address: message.address,
            encrypted_bal: bal_enc,
        };

        Ok(msg)
    }
}

fn decrypt_state(sk: &SigningKey, ciphertext: &[u8]) -> Result<State, Status> {
    let o =
        decrypt(&sk.to_bytes(), ciphertext).map_err(|e| Status::invalid_argument(e.to_string()))?;
    serde_json::from_slice(&o).map_err(|e| Status::invalid_argument(e.to_string()))
}

fn encrypt_balance(balance: Balance, ephemeral_pk: VerifyingKey) -> Result<HexBinary, Status> {
    let serialized_balance = serde_json::to_string(&balance).expect("infallible serializer");

    match encrypt(&ephemeral_pk.to_sec1_bytes(), serialized_balance.as_bytes()) {
        Ok(encrypted_balance) => Ok(encrypted_balance.into()),
        Err(e) => Err(Status::internal(format!("Encryption error: {}", e))),
    }
}
