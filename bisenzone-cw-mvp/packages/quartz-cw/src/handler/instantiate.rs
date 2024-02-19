use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use quartz_tee_ra::Error as RaVerificationError;

use crate::error::Error;
use crate::handler::Handler;
use crate::msg::execute::attested::{Attestation, EpidAttestation};
use crate::msg::instantiate::{CoreInstantiate, Instantiate};
use crate::state::Config;
use crate::state::CONFIG;

impl Handler for Instantiate<EpidAttestation> {
    fn handle(self, deps: DepsMut<'_>, env: &Env, info: &MessageInfo) -> Result<Response, Error> {
        if self.0.msg().mr_enclave() != self.0.attestation().mr_enclave() {
            return Err(RaVerificationError::MrEnclaveMismatch.into());
        }
        self.0.handle(deps, env, info)
    }
}

impl Handler for CoreInstantiate {
    fn handle(self, deps: DepsMut<'_>, _env: &Env, _info: &MessageInfo) -> Result<Response, Error> {
        CONFIG
            .save(deps.storage, &Config::new(self.mr_enclave()))
            .map_err(Error::Std)?;
        Ok(Response::new().add_attribute("action", "instantiate"))
    }
}
