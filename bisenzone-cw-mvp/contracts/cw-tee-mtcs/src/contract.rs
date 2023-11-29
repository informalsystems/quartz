use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
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
            compute_node_pub_key,
            nonce,
        }) => execute::enqueue_join_request(deps, compute_node_pub_key, nonce),
    }
}

pub mod execute {
    use cosmwasm_std::{DepsMut, Response};
    use ecies::PublicKey;

    use crate::state::Nonce;
    use crate::state::{Request, REQUESTS};
    use crate::ContractError;

    pub fn enqueue_join_request(
        deps: DepsMut,
        compute_node_pub_key: String,
        nonce: Nonce,
    ) -> Result<Response, ContractError> {
        let _ = PublicKey::parse_slice(compute_node_pub_key.as_bytes(), None)?;

        REQUESTS.save(
            deps.storage,
            &nonce,
            &Request::JoinComputeNode(compute_node_pub_key.clone()),
        )?;

        Ok(Response::new()
            .add_attribute("action", "enqueue_request")
            .add_attribute("compute_node_pub_key", compute_node_pub_key))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    todo!()
}

pub mod query {}

#[cfg(test)]
mod tests {}
