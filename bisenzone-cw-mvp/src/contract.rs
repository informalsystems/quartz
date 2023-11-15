use cosmwasm_std::{
    entry_point, to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE, UTILIZATION};

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
    use cosmwasm_std::Uint128;

    use super::*;

    pub fn upload_obligation(
        deps: DepsMut,
        info: MessageInfo,
        creditor: String,
        amount: Uint128,
        memo: String,
    ) -> Result<Response, ContractError> {
        let creditor = deps.api.addr_validate(&creditor)?;

        UTILIZATION.update(
            deps.storage,
            (&creditor, &info.sender),
            |utilization| -> Result<_, ContractError> {
                // Uncomment if we want to only allow ourselves to add obligations
                // if info.sender != state.owner {
                //     return Err(ContractError::Unauthorized);
                // }

                let utilization = utilization.unwrap_or_default() + amount;
                Ok(utilization)
            },
        )?;

        Ok(Response::new()
            .add_attribute("action", "upload_obligation")
            .add_attribute("memo", memo))
    }

    pub fn apply_cycle(
        deps: DepsMut,
        path: Vec<String>,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        let mut volume_cleared = Uint128::zero();

        let path = path
            .into_iter()
            .map(|addr| deps.api.addr_validate(&addr))
            .collect::<StdResult<Vec<Addr>>>()?;

        validate_cycle(&path, amount, &deps)?;

        for from_to in path.windows(2) {
            let (from, to) = (&from_to[0], &from_to[1]);

            UTILIZATION.update(
                deps.storage,
                (&to, &from),
                |utilization| -> Result<_, ContractError> {
                    let utilization = utilization.unwrap_or_default() - amount;
                    volume_cleared += amount;

                    Ok(utilization)
                },
            )?;
        }
        Ok(Response::new()
            .add_attribute("action", "apply_cycle")
            .add_attribute("volume_cleared", format!("{}", volume_cleared)))
    }

    fn validate_cycle(path: &[Addr], amount: Uint128, deps: &DepsMut) -> Result<(), ContractError> {
        if path.first() != path.last() {
            return Err(ContractError::PathNotCycle);
        }

        for from_to in path.windows(2) {
            let (from, to) = (&from_to[0], &from_to[1]);

            if amount > UTILIZATION.load(deps.storage, (to, from))? {
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
    use cosmwasm_std::Order;

    use crate::msg::GetObligationsResponse;
    use crate::state::UTILIZATION;

    pub fn get_obligations(deps: Deps, creditor: String) -> StdResult<GetObligationsResponse> {
        let creditor = deps.api.addr_validate(&creditor)?;

        let keys = UTILIZATION
            .keys(deps.storage, None, None, Order::Ascending)
            .collect::<StdResult<Vec<(Addr, Addr)>>>()?
            .into_iter()
            .filter(|(from, _)| from == creditor);
        Ok(GetObligationsResponse {
            obligations: keys
                .map(|(from, to)| {
                    let utilization = UTILIZATION.load(deps.storage, (&from, &to)).unwrap();
                    (to.to_string(), utilization)
                })
                .collect(),
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
