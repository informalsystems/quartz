use cosmwasm_std::{
    entry_point, to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Uint128,
};
use cw2::set_contract_version;
use cw20_base::{
    contract::{execute_mint, query_balance},
    state::{MinterData, TokenInfo, TOKEN_INFO},
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE, UTILIZATION};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-mtcs";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender.to_string(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    // store token info using cw20-base format
    let data = TokenInfo {
        name: "liquidity savings".to_string(),
        symbol: "!$".to_string(),
        decimals: 0,
        total_supply: Uint128::zero(),
        // set self as minter, so we can properly execute mint and burn
        mint: Some(MinterData {
            minter: env.contract.address,
            cap: None,
        }),
    };
    TOKEN_INFO.save(deps.storage, &data)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UploadObligation {
            creditor,
            amount,
            memo,
        } => execute::upload_obligation(deps, info, creditor, amount, memo),
        ExecuteMsg::ApplyCycle { path, amount } => {
            execute::apply_cycle(deps, env, info, path, amount)
        }
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
        env: Env,
        info: MessageInfo,
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
                (to, from),
                |utilization| -> Result<_, ContractError> {
                    let utilization = utilization.unwrap_or_default() - amount;
                    volume_cleared += amount;

                    Ok(utilization)
                },
            )?;
        }

        // call into cw20-base to mint the token, call as self as no one else is allowed
        let sub_info = MessageInfo {
            sender: env.contract.address.clone(),
            funds: vec![],
        };
        execute_mint(deps, env, sub_info, info.sender.to_string(), volume_cleared)?;

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
        QueryMsg::Balance { address } => to_json_binary(&query_balance(deps, address)?),
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
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{coins, from_json, OwnedDeps};
    use cw20::BalanceResponse;

    use crate::msg::GetObligationsResponse;

    const ALICE_ADDRESS: &str = "wasm19xlctyn7ha6pqg7pk9lnk8y60rk8646dm86qgv";
    const BOB_ADDRESS: &str = "wasm19u72czh0w4jraan8esalv48nrwemh8kgax69yw";
    const CHARLIE_ADDRESS: &str = "wasm12r9t5wmre89rwakr0e5nyhfmaf4kdleyltsm9f";

    #[test]
    fn test_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg;
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn test_upload_obligation() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg;
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        create_obligation(&mut deps, ALICE_ADDRESS, BOB_ADDRESS, 100, "alice -> bob");

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetObligations {
                creditor: BOB_ADDRESS.to_string(),
            },
        )
        .unwrap();
        let value: GetObligationsResponse = from_json(&res).unwrap();
        assert_eq!(&100u32.into(), value.obligations[0].1);
    }

    fn create_obligation(
        deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>,
        debtor: &str,
        creditor: &str,
        amount: u32,
        memo: &str,
    ) {
        let info = mock_info(debtor, &coins(2, "token"));
        let msg = ExecuteMsg::UploadObligation {
            creditor: creditor.to_string(),
            amount: amount.into(),
            memo: memo.to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    }

    #[test]
    fn test_apply_cycle() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg;
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        create_obligation(&mut deps, ALICE_ADDRESS, BOB_ADDRESS, 100, "alice -> bob");
        create_obligation(
            &mut deps,
            BOB_ADDRESS,
            CHARLIE_ADDRESS,
            80,
            "bob -> charlie",
        );
        create_obligation(
            &mut deps,
            CHARLIE_ADDRESS,
            ALICE_ADDRESS,
            70,
            "charlie -> alice",
        );

        let info = mock_info(ALICE_ADDRESS, &coins(2, "token"));
        let msg = ExecuteMsg::ApplyCycle {
            path: [ALICE_ADDRESS, BOB_ADDRESS, CHARLIE_ADDRESS, ALICE_ADDRESS]
                .into_iter()
                .map(ToString::to_string)
                .collect(),
            amount: 70u32.into(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Cycle should be cleared and only `30` should remain in `alice -> bob`
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetObligations {
                creditor: BOB_ADDRESS.to_string(),
            },
        )
        .unwrap();
        let value: GetObligationsResponse = from_json(&res).unwrap();
        assert_eq!(&30u32.into(), value.obligations[0].1);

        // Check that alice received her karma tokens
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Balance {
                address: ALICE_ADDRESS.to_string(),
            },
        )
        .unwrap();
        let value: BalanceResponse = from_json(&res).unwrap();
        assert_eq!(&210u32.into(), value.balance);
    }
}
