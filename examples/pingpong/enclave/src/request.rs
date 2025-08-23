use ecies::{decrypt, encrypt};
use ping_pong_contract::{
    msg::{execute, execute::Ping, AttestedMsg, ExecuteMsg},
    state::PINGS_KEY,
};
use quartz_common::{
    contract::msg::execute::attested::{HasUserData, RawNoop},
    enclave::{
        attestor::{Attestor, DefaultAttestor},
        handler::Handler,
        proof_of_publication::ProofOfPublication,
        store::Store,
        DefaultSharedEnclave, Enclave,
    },
};
use tonic::Status;

use crate::proto::PingRequest;

pub type EnclaveResponse = ExecuteMsg<<DefaultAttestor as Attestor>::RawAttestation>;

#[derive(Clone, Debug)]
pub enum EnclaveRequest {
    Ping(PingRequest),
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
impl Handler<DefaultSharedEnclave<()>> for EnclaveRequest {
    type Error = Status;
    type Response = EnclaveResponse;

    async fn handle(self, ctx: &DefaultSharedEnclave<()>) -> Result<Self::Response, Self::Error> {
        let attestor = ctx.attestor().await;
        match self {
            EnclaveRequest::Ping(request) => request
                .handle(ctx)
                .await
                .map(|msg| attested_msg(msg, attestor))?
                .map(ExecuteMsg::Pong),
        }
    }
}

#[async_trait::async_trait]
impl Handler<DefaultSharedEnclave<()>> for PingRequest {
    type Error = Status;
    type Response = execute::Pong;

    async fn handle(self, ctx: &DefaultSharedEnclave<()>) -> Result<Self::Response, Self::Error> {
        // verify proof
        let proof: ProofOfPublication<Ping> = {
            let message = self.message;
            serde_json::from_str(&message).map_err(|e| Status::invalid_argument(e.to_string()))?
        };
        let contract = ctx
            .store()
            .await
            .get_contract()
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("contract not found"))?;
        let config = ctx
            .store()
            .await
            .get_config()
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("config not found"))?;
        let (proof_value, ping) = proof
            .verify(
                config.light_client_opts(),
                contract,
                PINGS_KEY.to_string(),
                None,
            )
            .map_err(Status::failed_precondition)?;

        let proof_value_matches_msg =
            serde_json::to_string(&ping.message).is_ok_and(|s| s.as_bytes() == proof_value);
        if !proof_value_matches_msg {
            return Err(Status::failed_precondition("proof verification"));
        }

        // Perform enclave logic
        // Decrypt the ciphertext using enclave private key
        let decrypted_message: String = {
            let sk = ctx.key_manager().await.read_lock().await.sk.clone();

            let msg_bytes = decrypt(&sk.to_bytes(), &ping.message)
                .map_err(|_| Status::invalid_argument("decryption failed"))?;

            String::from_utf8(msg_bytes)
                .map_err(|_| Status::invalid_argument("Byte conversion to string failed"))?
        };

        println!("\nDecryption Result: {}\n", decrypted_message);

        // Encrypt enclave response to the user's provided pubkey
        let response: Vec<u8> = {
            let response = format!("Enclave responded to {}", decrypted_message);
            encrypt(&ping.pubkey, response.as_bytes())
                .map_err(|e| Status::invalid_argument(e.to_string()))?
        };

        // Prepare message to chain
        let msg = execute::Pong {
            pubkey: ping.pubkey,
            response: response.into(),
        };

        Ok(msg)
    }
}
