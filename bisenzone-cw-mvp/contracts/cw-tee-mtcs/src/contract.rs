use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::execute::{BootstrapKeyManagerMsg, JoinComputeNodeMsg, RegisterEpochKeyMsg};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-tee-mtcs";
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
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::BootstrapKeyManager(BootstrapKeyManagerMsg {
            compute_mrenclave,
            key_manager_mrenclave,
            tcb_info,
        }) => {
            execute::bootstrap_key_manger(deps, compute_mrenclave, key_manager_mrenclave, tcb_info)
        }
        ExecuteMsg::RegisterEpochKey(RegisterEpochKeyMsg { epoch_key }) => {
            execute::register_epoch_key(deps, epoch_key)
        }
        ExecuteMsg::JoinComputeNode(JoinComputeNodeMsg {
            io_exchange_key,
            address,
            nonce,
        }) => execute::enqueue_join_request(deps, io_exchange_key, address, nonce),
    }
}

pub mod execute {
    use cosmwasm_std::{DepsMut, Response};
    use k256::ecdsa::VerifyingKey;

    use crate::state::{
        EpochState, Mrenclave, RawAddress, RawMrenclave, RawNonce, RawPublicKey, RawTcbInfo,
        SgxState, EPOCH_STATE, SGX_STATE,
    };
    use crate::state::{Request, REQUESTS};
    use crate::ContractError;
    use crate::ContractError::BadLength;

    pub fn bootstrap_key_manger(
        deps: DepsMut,
        compute_mrenclave: RawMrenclave,
        key_manager_mrenclave: RawMrenclave,
        tcb_info: RawTcbInfo,
    ) -> Result<Response, ContractError> {
        let _: Mrenclave = hex::decode(&compute_mrenclave)?
            .try_into()
            .map_err(|_| BadLength)?;
        let _: Mrenclave = hex::decode(&key_manager_mrenclave)?
            .try_into()
            .map_err(|_| BadLength)?;
        // TODO(hu55a1n1): validate TcbInfo

        let sgx_state = SgxState {
            compute_mrenclave: compute_mrenclave.clone(),
            key_manager_mrenclave: key_manager_mrenclave.clone(),
            tcb_info: tcb_info.clone(),
        };

        if SGX_STATE.exists(deps.storage) {
            return Err(ContractError::Unauthorized);
        }

        SGX_STATE.save(deps.storage, &sgx_state)?;

        Ok(Response::new()
            .add_attribute("action", "bootstrap_key_manger")
            .add_attribute("compute_mrenclave", compute_mrenclave)
            .add_attribute("key_manager_mrenclave", key_manager_mrenclave)
            .add_attribute("tcb_info", tcb_info))
    }

    pub fn register_epoch_key(
        deps: DepsMut,
        epoch_key: RawPublicKey,
    ) -> Result<Response, ContractError> {
        let _ = VerifyingKey::from_sec1_bytes(&hex::decode(&epoch_key)?)?;

        let epoch_state = EpochState {
            epoch_key: epoch_key.clone(),
        };
        EPOCH_STATE.save(deps.storage, &epoch_state)?;

        Ok(Response::new()
            .add_attribute("action", "register_epoch_key")
            .add_attribute("epoch_key", epoch_key))
    }

    pub fn enqueue_join_request(
        deps: DepsMut,
        io_exchange_key: RawPublicKey,
        address: RawAddress,
        nonce: RawNonce,
    ) -> Result<Response, ContractError> {
        let _ = VerifyingKey::from_sec1_bytes(&hex::decode(&io_exchange_key)?)?;
        let _ = deps.api.addr_validate(&address)?;
        let _ = hex::decode(&nonce);

        let mut requests = REQUESTS.may_load(deps.storage)?.unwrap_or_default();
        requests.push((
            nonce,
            Request::JoinComputeNode((io_exchange_key.clone(), address)),
        ));
        REQUESTS.save(deps.storage, &requests)?;

        Ok(Response::new()
            .add_attribute("action", "enqueue_request")
            .add_attribute("io_exchange_key", io_exchange_key))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetSgxState {} => to_json_binary(&query::get_sgx_state(deps)?),
        QueryMsg::GetEpochState {} => to_json_binary(&query::get_epoch_state(deps)?),
        QueryMsg::GetRequests {} => to_json_binary(&query::get_requests(deps)?),
    }
}

pub mod query {
    use cosmwasm_std::{Deps, StdResult};

    use crate::msg::query::{GetEpochStateResponse, GetRequestsResponse, GetSgxStateResponse};
    use crate::state::{EpochState, SgxState, EPOCH_STATE, REQUESTS, SGX_STATE};

    pub fn get_sgx_state(deps: Deps) -> StdResult<GetSgxStateResponse> {
        let SgxState {
            compute_mrenclave,
            key_manager_mrenclave,
            ..
        } = SGX_STATE.load(deps.storage)?;
        Ok(GetSgxStateResponse {
            compute_mrenclave,
            key_manager_mrenclave,
        })
    }

    pub fn get_epoch_state(deps: Deps) -> StdResult<GetEpochStateResponse> {
        let EpochState { epoch_key } = EPOCH_STATE.load(deps.storage)?;
        Ok(GetEpochStateResponse { epoch_key })
    }

    pub fn get_requests(deps: Deps) -> StdResult<GetRequestsResponse> {
        Ok(GetRequestsResponse {
            requests: REQUESTS.load(deps.storage)?,
        })
    }
}

#[cfg(test)]
mod tests {}
