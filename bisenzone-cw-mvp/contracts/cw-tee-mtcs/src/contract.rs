use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::execute::{BootstrapKeyManagerMsg, JoinComputeNodeMsg};
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
        Mrenclave, RawAddress, RawMrenclave, RawNonce, RawPublicKey, RawTcbInfo, SgxState,
        SGX_STATE,
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

    pub fn enqueue_join_request(
        deps: DepsMut,
        io_exchange_key: RawPublicKey,
        address: RawAddress,
        nonce: RawNonce,
    ) -> Result<Response, ContractError> {
        let _ = VerifyingKey::from_sec1_bytes(&hex::decode(&io_exchange_key)?)?;
        let _ = deps.api.addr_validate(&address)?;
        let _ = hex::decode(&nonce);

        REQUESTS.save(
            deps.storage,
            &nonce,
            &Request::JoinComputeNode((io_exchange_key.clone(), address)),
        )?;

        Ok(Response::new()
            .add_attribute("action", "enqueue_request")
            .add_attribute("io_exchange_key", io_exchange_key))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetSgxState {} => to_json_binary(&query::get_sgx_state(deps)?),
        QueryMsg::GetRequests {} => to_json_binary(&query::get_requests(deps)?),
    }
}

pub mod query {
    use cosmwasm_std::{Deps, Order, StdResult};

    use crate::msg::query::{GetRequestsResponse, GetSgxStateResponse};
    use crate::state::{RawNonce, Request, SgxState, REQUESTS, SGX_STATE};

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

    pub fn get_requests(deps: Deps) -> StdResult<GetRequestsResponse> {
        Ok(GetRequestsResponse {
            requests: REQUESTS
                .range(deps.storage, None, None, Order::Ascending)
                .collect::<StdResult<Vec<(RawNonce, Request)>>>()?,
        })
    }
}

#[cfg(test)]
mod tests {}
