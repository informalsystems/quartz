#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use quartz_dcap_verifier_msgs::{ExecuteMsg, InstantiateMsg, QueryMsg};
use quartz_tee_ra::{
    intel_sgx::dcap::{Collateral, Quote3, TrustedIdentity},
    verify_dcap_attestation, Error,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, StdError> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> StdResult<Response> {
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::VerifyDcapAttestation {
            quote,
            collateral,
            identities,
        } => {
            let quote = Quote3::<Vec<u8>>::try_from(Vec::<u8>::from(quote))
                .map_err(|e| StdError::generic_err(format!("Quote parse error: {e}")))?;
            let collateral: Collateral = ciborium::from_reader(collateral.as_slice())
                .map_err(|e| StdError::generic_err(format!("Collateral deserialize error: {e}")))?;
            let identities: Vec<TrustedIdentity> = if let Some(identities) = identities {
                ciborium::from_reader(identities.as_slice())
                    .map_err(|e| StdError::generic_err(format!("Identities parse error: {e}")))?
            } else {
                vec![]
            };

            // attestation handler MUST verify that the user_data and mr_enclave match the config/msg
            let verification_output =
                verify_dcap_attestation(quote, collateral, identities.as_slice());

            // attestation handler MUST verify that the user_data and mr_enclave match the config/msg
            if verification_output.is_success().into() {
                to_json_binary(&())
            } else {
                Err(StdError::generic_err(
                    Error::Dcap(Box::new(verification_output)).to_string(),
                ))
            }
        }
    }
}
