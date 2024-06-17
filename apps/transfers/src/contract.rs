use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128, BankMsg, coins
};

use cw2::set_contract_version;
use cw20_base::{
    contract::{execute_mint, query_balance as cw20_query_balance},
    state::{MinterData, TokenInfo, TOKEN_INFO},
};
use quartz_cw::{handler::RawHandler, state::EPOCH_COUNTER};

use crate::{
    error::ContractError,
    msg::{
        execute::{SubmitObligationMsg, SubmitObligationsMsg, SubmitSetoffsMsg},
        ExecuteMsg, InstantiateMsg, QueryMsg,
    },
    state::{
        current_epoch_key, LiquiditySourcesItem, ObligationsItem, State, LIQUIDITY_SOURCES_KEY,
        OBLIGATIONS_KEY, STATE,
    },
};


#[cfg_attr(entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // must be the handled first!
    msg.quartz.handle_raw(deps.branch(), &env, &info)?;

    DENOM.save(deps.storage, msg.denom);

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(entry_point)]
pub fn execute(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use execute::*;

    match msg {
        ExecuteMsg::Quartz(msg) => msg.handle_raw(deps, &env, &info).map_err(Into::into),
        ExecuteMsg::TransferRequest(msg) => transfer_request(deps, env, info, msg)
        ExecuteMsg::Update(msg) => update(deps, env, info, msg)
        ExecuteMsg::Deposit() => deposit(deps, env, info)
        ExecuteMsg::Withdraw() => withdraw(deps, env, info)
    }
}

pub mod execute {
    use std::collections::BTreeMap;

    use cosmwasm_std::{DepsMut, Env, HexBinary, MessageInfo, Response, StdResult};
    use cw20_base::contract::{execute_burn, execute_mint};
    use quartz_cw::state::{Hash, EPOCH_COUNTER};


    pub fn transfer_request(deps: DepsMut, env: Env, info: MessageInfo, msg: TransferRequestMsg) -> Result<Response, StdError> {
        let mut requests = REQUESTS.load(deps.storage);

        requests.append(state::Request::Ciphertext(msg.ciphertext));

        REQUESTS.save(deps.storage, requests);

        Ok(Response::new())
    }

    pub fn update(deps: DepsMut, env: Env, info: MessageInfo, msg: UpdateMsg) -> Result<Response, StdError> {
        //TODO: validate

        // Store state
        STATE.save(deps.storage, msg.ciphertext);

        // Clear queue
        let mut requests: Vec<state::Request> = REQUESTS.load(deps.storage);

        let requests = requests.drain(0..msg.quantity).collect();

        REQUESTS.save(deps.storage, requests);

        // Process withdrawals
        let denom = DENOM.load(deps.storage)?;

        let messages = msg.withdrawals.into_iter().map(|(user, funds)| BankMsg::Send {
            to_address: user.to_string(),
            amount: coins(funds, &denom),
        });    

        let resp = Response::new()
            .add_messages(messages);
        
        Ok(resp)
    }

    pub fn deposit(deps: DepsMut, env: Env, info: MessageInfo) ->  Result<Response, StdError> {
        let denom = DENOM.load(deps.storage)?;
        let quantity = cw_utils::must_pay(&info, &denom)?.u128();

        let mut requests = REQUESTS.load(deps.storage);

        requests.append(state::Request::Deposit(info.sender, quantity));

        REQUESTS.save(deps.storage, requests);
        
        Ok(Response::new())
    }

    pub fn withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, StdError> {

        let mut requests = REQUESTS.load(deps.storage);

        requests.append(state::Request::Withdraw(info.sender));

        REQUESTS.save(deps.storage, requests);
        
    }
} 
