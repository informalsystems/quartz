use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use cw2::set_contract_version;
use quartz_tee_ra::{
    intel_sgx::dcap::{Collateral, Quote3, TrustedIdentity},
    verify_dcap_attestation, Error,
};

use crate::{
    error::{into_std_err, ContractError},
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{State, STATE},
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:quartz-dcap-verifier";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender.to_string(),
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
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
            let quote = Quote3::<Vec<u8>>::try_from(quote).map_err(into_std_err)?;
            let collateral: Collateral =
                serde_json::from_value(collateral).map_err(into_std_err)?;
            let identities: Vec<TrustedIdentity> =
                serde_json::from_value(identities).map_err(into_std_err)?;

            // attestation handler MUST verify that the user_data and mr_enclave match the config/msg
            let verification_output =
                verify_dcap_attestation(quote, collateral, identities.as_slice());

            // attestation handler MUST verify that the user_data and mr_enclave match the config/msg
            if verification_output.is_success().into() {
                Ok(Binary::default())
            } else {
                Err(StdError::generic_err(
                    Error::Dcap(verification_output).to_string(),
                ))
            }
        }
    }
}
