use cosmwasm_std::{
    from_json, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, QueryRequest, Response,
    StdResult, WasmQuery,
};
use quartz_tee_ra::{
    intel_sgx::dcap::TrustedMrEnclaveIdentity, verify_dcap_attestation, verify_epid_attestation,
    Error as RaVerificationError,
};
use serde_json::Value;

use crate::{
    error::Error,
    handler::Handler,
    msg::execute::attested::{
        Attestation, Attested, AttestedMsgSansHandler, DcapAttestation, EpidAttestation,
        GetTcbInfoResponse, HasUserData, MockAttestation, TcbInfoQueryMsg,
    },
    state::CONFIG,
};

pub fn query_tcbinfo(deps: Deps<'_>, tcbinfo_addr: String, fmspc: String) -> StdResult<Binary> {
    let query_msg = TcbInfoQueryMsg::GetInfo { fmspc };

    let request = QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: tcbinfo_addr,
        msg: to_json_binary(&query_msg)?,
    });

    deps.querier.query(&request)
}

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
                .expect("tcbInfo not found in JSON")
                .get("fmspc")
                .expect("fmspc not found in tcbInfo")
                .as_str()
                .expect("fmspc is not a string"),
        )
        .expect("failed to decode fmspc hex string");

        let fmspc: [u8; 6] = fmspc_raw.try_into().expect("fmspc should be 6 bytes");
        let fmspc_value = u16::from_be_bytes([fmspc[4], fmspc[5]]);
        let fmspc_hex = format!("{:04X}", fmspc_value);

        // We need to get the CONFIG
        let rawconfig = CONFIG.load(deps.storage)?;
        // We retrieve the contract address
        let tcbinfo_addr = rawconfig.tcb_info();

        let tcb_info_response = query_tcbinfo(deps.as_ref(), tcbinfo_addr, fmspc_hex)?;

        let _tcb_info: GetTcbInfoResponse = from_json(tcb_info_response)?;

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
