use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use quartz_tee_ra::Error as RaVerificationError;

use crate::error::Error;
use crate::handler::Handler;
use crate::msg::execute::attested::{Attestation, EpidAttestation, MockAttestation};
use crate::msg::instantiate::{CoreInstantiate, Instantiate};
use crate::state::CONFIG;
use crate::state::{RawConfig, EPOCH_COUNTER};

impl Handler for Instantiate<EpidAttestation> {
    fn handle(self, deps: DepsMut<'_>, env: &Env, info: &MessageInfo) -> Result<Response, Error> {
        if self.0.msg().config().mr_enclave() != self.0.attestation().mr_enclave() {
            return Err(RaVerificationError::MrEnclaveMismatch.into());
        }
        self.0.handle(deps, env, info)
    }
}

impl Handler for Instantiate<MockAttestation> {
    fn handle(self, deps: DepsMut<'_>, env: &Env, info: &MessageInfo) -> Result<Response, Error> {
        self.0.handle(deps, env, info)
    }
}

impl Handler for CoreInstantiate {
    fn handle(self, deps: DepsMut<'_>, _env: &Env, _info: &MessageInfo) -> Result<Response, Error> {
        CONFIG
            .save(deps.storage, &RawConfig::from(self.config().clone()))
            .map_err(Error::Std)?;

        EPOCH_COUNTER.save(deps.storage, &1).map_err(Error::Std)?;

        Ok(Response::new().add_attribute("action", "instantiate"))
    }
}
