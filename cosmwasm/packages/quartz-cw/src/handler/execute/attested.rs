use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use quartz_tee_ra::{
    intel_sgx::dcap::TrustedMrEnclaveIdentity, verify_dcap_attestation, verify_epid_attestation,
    Error as RaVerificationError,
};
use serde_json::Value;
use tcbinfo::contract::query::get_info;

use crate::{
    error::Error,
    handler::Handler,
    msg::execute::attested::{
        Attestation, Attested, AttestedMsgSansHandler, DcapAttestation, EpidAttestation,
        HasUserData, MockAttestation,
    },
    state::CONFIG,
};

impl Handler for EpidAttestation {
    fn handle(
        self,
        _deps: DepsMut<'_>,
        _env: &Env,
        _info: &MessageInfo,
    ) -> Result<Response, Error> {
        // attestation handler MUST verify that the user_data and mr_enclave match the config/msg
        verify_epid_attestation(
            self.clone().into_report(),
            self.mr_enclave(),
            self.user_data(),
        )
        .map(|_| Response::default())
        .map_err(Error::RaVerification)
    }
}

impl Handler for DcapAttestation {
    fn handle(self, deps: DepsMut<'_>, _env: &Env, _info: &MessageInfo) -> Result<Response, Error> {
        let (quote, collateral) = self.clone().into_tuple();
        let mr_enclave = TrustedMrEnclaveIdentity::new(self.mr_enclave().into(), [""; 0], [""; 0]);

        // Retrieve the FMSPC from the collateral
        let fmspc = collateral.tcb_info();

        let tcb_info_json: Value = serde_json::from_str(fmspc).expect("could not read tcbinfo");
        let fmspc_raw = hex::decode(
            tcb_info_json
                .get("tcbInfo")
                .unwrap()
                .get("fmspc")
                .unwrap()
                .as_str()
                .expect("could not find fmspc string"),
        )
        .expect("failed to decode fmspc hex string");

        let fmspc: [u8; 6] = fmspc_raw.try_into().unwrap();
        let fmspc_value = u16::from_be_bytes([fmspc[4], fmspc[5]]);
        let fmspc_hex = format!("{:04X}", fmspc_value);

        // @dusterbloom not sure about this part
        let _tcb_info_response =
            get_info(deps.as_ref(), fmspc_hex).map_err(|_| Error::TcbInfoQueryError)?;

        // attestation handler MUST verify that the user_data and mr_enclave match the config/msg
        let verification_output = verify_dcap_attestation(quote, collateral, &[mr_enclave.into()]);

        // attestation handler MUST verify that the user_data and mr_enclave match the config/msg
        if verification_output.is_success().into() {
            Ok(Response::default())
        } else {
            Err(Error::RaVerification(RaVerificationError::Dcap(
                verification_output,
            )))
        }
    }
}

impl Handler for MockAttestation {
    fn handle(
        self,
        _deps: DepsMut<'_>,
        _env: &Env,
        _info: &MessageInfo,
    ) -> Result<Response, Error> {
        Ok(Response::default())
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
            // in such cases, the InstantiateMsg handler will verify that the mr_enclave matches
            if config.mr_enclave() != attestation.mr_enclave() {
                return Err(RaVerificationError::MrEnclaveMismatch.into());
            }
        }

        // handle message first, this has 2 benefits -
        // 1. we avoid (the more expensive) attestation verification if the message handler fails
        // 2. we allow the message handler to make changes to the config so that the attestation
        //    handler can use those changes, e.g. InstantiateMsg
        // return response from msg handle to include pub_key attribute
        let res_msg = Handler::handle(msg, deps.branch(), env, info)?;
        let res_attest = Handler::handle(attestation, deps, env, info)?;

        Ok(res_msg
            .add_events(res_attest.events)
            .add_attributes(res_attest.attributes))
    }
}

impl<T> Handler for AttestedMsgSansHandler<T> {
    fn handle(
        self,
        _deps: DepsMut<'_>,
        _env: &Env,
        _info: &MessageInfo,
    ) -> Result<Response, Error> {
        Ok(Response::default())
    }
}
