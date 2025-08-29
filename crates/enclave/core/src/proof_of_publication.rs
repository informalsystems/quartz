use std::time::Duration;

use cosmrs::AccountId;
use quartz_contract_core::state::LightClientOpts;
use quartz_cw_proof::proof::{
    cw::{CwProof, RawCwProof},
    key::CwAbciKey,
    Proof,
};
use quartz_tm_stateless_verifier::make_provider;
use serde::{Deserialize, Serialize};
use tendermint::{block::Height, Hash};
use tendermint_light_client::{
    light_client::Options,
    types::{LightBlock, TrustThreshold},
};

/// A proof of publication for a given message/request.
///
/// This structure provides evidence that a message was published on-chain by combining:
///
/// - A **light client proof**
/// - A **Merkle proof**
/// - The original **message** that is claimed to have been published.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProofOfPublication<M> {
    light_client_proof: Vec<LightBlock>,
    merkle_proof: RawCwProof,
    msg: M,
}

impl<M> ProofOfPublication<M> {
    pub fn verify(
        self,
        light_client_opts: &LightClientOpts,
        trusted_height: Height,
        trusted_hash: Hash,
        contract_address: AccountId,
        storage_key: String,
        storage_namespace: Option<String>,
    ) -> Result<(Vec<u8>, M), String> {
        let config_trust_threshold = light_client_opts.trust_threshold();
        let trust_threshold =
            TrustThreshold::new(config_trust_threshold.0, config_trust_threshold.1).unwrap();

        let config_trusting_period = light_client_opts.trusting_period();
        let trusting_period = Duration::from_secs(config_trusting_period);

        let config_clock_drift = light_client_opts.max_clock_drift();
        let clock_drift = Duration::from_secs(config_clock_drift);
        let options = Options {
            trust_threshold,
            trusting_period,
            clock_drift,
        };

        let target_height = self.light_client_proof.last().unwrap().height();

        let primary_block = make_provider(
            light_client_opts.chain_id(),
            trusted_height,
            trusted_hash,
            self.light_client_proof,
            options,
        )
        .and_then(|mut primary| primary.verify_to_height(target_height))
        .map_err(|e| e.to_string())?;

        let key = CwAbciKey::new(contract_address, storage_key, storage_namespace);
        if key.into_vec() != self.merkle_proof.key() {
            return Err("Merkle proof key mismatch".to_string());
        }

        let proof = CwProof::from(self.merkle_proof);
        proof
            .verify(
                primary_block
                    .signed_header
                    .header
                    .app_hash
                    .as_bytes()
                    .to_vec(),
            )
            .map_err(|e| e.to_string())?;

        Ok((proof.value, self.msg))
    }

    pub fn target_height_hash(&self) -> (Height, Hash) {
        let proof_last_block = self.light_client_proof.last().unwrap();
        let target_height = proof_last_block.height();
        let target_hash = proof_last_block.signed_header.header().hash();

        (target_height, target_hash)
    }
}
