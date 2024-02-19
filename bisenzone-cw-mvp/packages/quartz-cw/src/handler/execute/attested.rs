use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use quartz_tee_ra::{verify_epid_attestation, Error as RaVerificationError};

use crate::error::Error;
use crate::handler::Handler;
use crate::msg::execute::attested::{Attestation, Attested, EpidAttestation, HasUserData};
use crate::state::CONFIG;

impl Handler for EpidAttestation {
    fn handle(
        self,
        _deps: DepsMut<'_>,
        _env: &Env,
        _info: &MessageInfo,
    ) -> Result<Response, Error> {
        let (report, mr_enclave, user_data) = self.into_tuple();
        verify_epid_attestation(report, mr_enclave, user_data)
            .map(|_| Response::default())
            .map_err(Error::RaVerification)
    }
}

impl<M, A> Handler for Attested<M, A>
where
    M: Handler + HasUserData,
    A: Handler + HasUserData + Attestation,
{
    fn handle(
        self,
        mut deps: DepsMut<'_>,
        env: &Env,
        info: &MessageInfo,
    ) -> Result<Response, Error> {
        let (msg, attestation) = self.into_tuple();
        if msg.user_data() != attestation.user_data() {
            return Err(RaVerificationError::UserDataMismatch.into());
        }

        if let Some(config) = CONFIG.may_load(deps.storage)? {
            // if we weren't able to load then the context was from InstantiateMsg so we don't fail
            if *config.mr_enclave() != attestation.mr_enclave() {
                return Err(RaVerificationError::MrEnclaveMismatch.into());
            }
        }

        Handler::handle(attestation, deps.branch(), env, info)?;
        Handler::handle(msg, deps, env, info)
    }
}
