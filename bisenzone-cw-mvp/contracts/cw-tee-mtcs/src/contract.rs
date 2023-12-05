use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::execute::JoinComputeNodeMsg;
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

    use crate::state::{RawAddress, RawNonce, RawPublicKey};
    use crate::state::{Request, REQUESTS};
    use crate::ContractError;

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
        QueryMsg::GetRequests {} => to_json_binary(&query::get_requests(deps)?),
    }
}

pub mod query {
    use cosmwasm_std::{Deps, Order, StdResult};

    use crate::msg::query::GetRequestsResponse;
    use crate::state::{RawNonce, Request, REQUESTS};

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
