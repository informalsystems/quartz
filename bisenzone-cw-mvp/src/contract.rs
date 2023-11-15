use cosmwasm_std::{
    entry_point, to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;
use itertools::Itertools;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-mtcs";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        utilization: Default::default(),
        owner: info.sender.clone(),
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
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UploadObligation {
            creditor,
            amount,
            memo,
        } => execute::upload_obligation(deps, info, creditor, amount, memo),
        ExecuteMsg::ApplyCycle { path, amount } => execute::apply_cycle(deps, path, amount),
    }
}

pub mod execute {
    use super::*;

    pub fn upload_obligation(
        deps: DepsMut,
        info: MessageInfo,
        creditor: Addr,
        amount: u64,
        memo: String,
    ) -> Result<Response, ContractError> {
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            // Uncomment if we want to only allow ourselves to add obligations
            // if info.sender != state.owner {
            //     return Err(ContractError::Unauthorized);
            // }

            *state
                .utilization
                .get_mut(&creditor)
                .unwrap()
                .get_mut(&info.sender)
                .unwrap() += amount;
            Ok(state)
        })?;

        Ok(Response::new()
            .add_attribute("action", "upload_obligation")
            .add_attribute("memo", memo))
    }

    pub fn apply_cycle(
        deps: DepsMut,
        path: Vec<Addr>,
        amount: u64,
    ) -> Result<Response, ContractError> {
        let mut volume_cleared = 0;

        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            validate_cycle(&path, amount, &state)?;

            for (from, to) in path.into_iter().tuples() {
                *state
                    .utilization
                    .get_mut(&to)
                    .unwrap()
                    .get_mut(&from)
                    .unwrap() -= amount;
                volume_cleared += amount;
            }

            Ok(state)
        })?;
        Ok(Response::new()
            .add_attribute("action", "apply_cycle")
            .add_attribute("volume_cleared", format!("{}", volume_cleared)))
    }

    fn validate_cycle(path: &[Addr], amount: u64, state: &State) -> Result<(), ContractError> {
        if path.first() != path.last() {
            return Err(ContractError::PathNotCycle);
        }

        for (from, to) in path.iter().tuples() {
            if amount > state.utilization[to][from] {
                return Err(ContractError::ClearingTooMuch);
            }
        }

        Ok(())
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetObligations { creditor } => {
            to_json_binary(&query::get_obligations(deps, creditor)?)
        }
    }
}

pub mod query {
    use super::*;

    use crate::msg::GetObligationsResponse;

    pub fn get_obligations(deps: Deps, creditor: Addr) -> StdResult<GetObligationsResponse> {
        let state = STATE.load(deps.storage)?;
        Ok(GetObligationsResponse {
            obligations: state.utilization[&creditor].clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_json};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_json(&res).unwrap();
        assert_eq!(17, value.count);
    }

    #[test]
    fn increment() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Increment {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_json(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        // should now be 5
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_json(&res).unwrap();
        assert_eq!(5, value.count);
    }
}
