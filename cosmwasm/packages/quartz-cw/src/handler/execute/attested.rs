use cosmwasm_std::{
    from_json, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, QueryRequest, Response,
    StdResult, WasmQuery,
};
use quartz_dcap_verifier_msgs::QueryMsg as DcapVerifierQueryMsg;
use quartz_tee_ra::{
    intel_sgx::dcap::{Collateral, TrustedIdentity, TrustedMrEnclaveIdentity},
    verify_epid_attestation, Error as RaVerificationError,
};
use serde::Serialize;
use tcbinfo_msgs::{GetTcbInfoResponse, QueryMsg as TcbInfoQueryMsg};

use crate::{
    error::Error,
    handler::Handler,
    msg::execute::attested::{
        Attestation, Attested, AttestedMsgSansHandler, DcapAttestation, EpidAttestation,
        HasUserData, MockAttestation, Quote,
    },
    state::CONFIG,
};

fn query_contract(
    deps: Deps<'_>,
    contract_addr: String,
    query_msg: impl Serialize,
) -> StdResult<Binary> {
    let request = QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr,
        msg: to_json_binary(&query_msg)?,
    });

    deps.querier.query(&request)
}

fn query_tcbinfo(deps: Deps<'_>, fmspc: String) -> Result<Binary, Error> {
    let tcbinfo_addr = {
        let config = CONFIG.load(deps.storage).map_err(Error::Std)?;
        config
            .tcbinfo_contract()
            .expect("TcbInfo contract address is required for DCAP")
            .to_string()
    };

    let fmspc_bytes =
        hex::decode(&fmspc).map_err(|_| Error::InvalidFmspc("Invalid FMSPC format".to_string()))?;
    if fmspc_bytes.len() != 6 {
        return Err(Error::InvalidFmspc("FMSPC must be 6 bytes".to_string()));
    }

    let query_msg = TcbInfoQueryMsg::GetTcbInfo { fmspc };

    query_contract(deps, tcbinfo_addr, &query_msg)
        .map_err(|err| Error::TcbInfoQueryError(err.to_string()))
}

fn query_dcap_verifier(
    deps: Deps<'_>,
    quote: Quote,
    mr_enclave: impl Into<TrustedIdentity>,
    updated_collateral: Collateral,
) -> Result<Binary, Error> {
    let query_msg = DcapVerifierQueryMsg::VerifyDcapAttestation {
        quote: quote.as_ref().to_vec(),
        collateral: serde_json::to_value(&updated_collateral).expect("infallible serializer"),
        identities: serde_json::to_value(&[mr_enclave.into()]).expect("infallible serializer"),
    };

    let dcap_verifier_contract = {
        let config = CONFIG.load(deps.storage).map_err(Error::Std)?;
        config
            .dcap_verifier_contract()
            .expect("verifier_contract address is required for DCAP")
            .to_string()
    };

    query_contract(deps, dcap_verifier_contract, &query_msg)
        .map_err(|err| Error::DcapVerificationQueryError(err.to_string()))
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
        let fmspc_hex = collateral.tcb_info().to_string();

        // Query the tcbinfo contract with the FMSPC retrieved and validated
        let tcb_info_query = query_tcbinfo(deps.as_ref(), fmspc_hex)?;
        let tcb_info_response: GetTcbInfoResponse = from_json(tcb_info_query)?;

        // Serialize the existing collateral
        let mut collateral_json: serde_json::Value =
            serde_json::to_value(&collateral).map_err(|e| {
                Error::TcbInfoQueryError(format!("Failed to serialize collateral: {}", e))
            })?;

        // Update the tcb_info in the serialized data
        collateral_json["tcb_info"] = tcb_info_response.tcb_info;

        // Deserialize back into a Collateral
        let updated_collateral: Collateral =
            serde_json::from_value(collateral_json).map_err(|e| {
                Error::TcbInfoQueryError(format!("Failed to deserialize updated collateral: {}", e))
            })?;

        query_dcap_verifier(deps.as_ref(), quote, mr_enclave, updated_collateral)
            .map(|_| Response::default())
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
