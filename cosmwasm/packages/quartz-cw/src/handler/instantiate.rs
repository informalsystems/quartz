use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, Uint64};
use quartz_tee_ra::Error as RaVerificationError;

use crate::{
    error::Error,
    handler::Handler,
    msg::{
        execute::attested::{Attestation, HasUserData},
        instantiate::{CoreInstantiate, Instantiate},
    },
    state::{RawConfig, CONFIG, EPOCH_COUNTER},
};

impl<A> Handler for Instantiate<A>
where
    A: Attestation + Handler + HasUserData,
{
    fn handle(self, deps: DepsMut<'_>, env: &Env, info: &MessageInfo) -> Result<Response, Error> {
        if self.0.msg().config().mr_enclave() != self.0.attestation().mr_enclave() {
            return Err(RaVerificationError::MrEnclaveMismatch.into());
        }
        self.0.handle(deps, env, info)
    }
}

impl Handler for CoreInstantiate {
    fn handle(self, deps: DepsMut<'_>, _env: &Env, _info: &MessageInfo) -> Result<Response, Error> {
        CONFIG
            .save(deps.storage, &RawConfig::from(self.config().clone()))
            .map_err(Error::Std)?;
        let epoch_counter = Uint64::new(1);

        EPOCH_COUNTER
            .save(deps.storage, &epoch_counter)
            .map_err(Error::Std)?;

        Ok(Response::new().add_attribute("action", "instantiate"))
    }
}
