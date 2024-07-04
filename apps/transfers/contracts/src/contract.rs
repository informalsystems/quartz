use cosmwasm_std::{entry_point, DepsMut, Env, HexBinary, MessageInfo, Response};
use quartz_cw::handler::RawHandler;

use crate::{
    error::ContractError,
    msg::{
        execute::{QueryResponseMsg, Request, UpdateMsg},
        ExecuteMsg, InstantiateMsg,
    },
    state::{DENOM, REQUESTS, STATE},
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // must be handled first!
    msg.quartz.handle_raw(deps.branch(), &env, &info)?;

    DENOM.save(deps.storage, &msg.denom)?;

    let requests: Vec<Request> = Vec::new();
    REQUESTS.save(deps.storage, &requests)?;

    let state: HexBinary = HexBinary::from(&[0x00]);
    STATE.save(deps.storage, &state)?;

    // TODO - do I need to instantiate  BALANCES

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use execute::*;

    match msg {
        // Quartz msgs
        ExecuteMsg::Quartz(msg) => msg.handle_raw(deps, &env, &info).map_err(Into::into),

        // Clear user msgs
        ExecuteMsg::Deposit => deposit(deps, env, info),
        ExecuteMsg::Withdraw => withdraw(deps, env, info),
        ExecuteMsg::ClearTextTransferRequest(_) => unimplemented!(),
        ExecuteMsg::QueryRequest(msg) => query_balance(deps, env, info, msg),

        // Cipher user msgs
        ExecuteMsg::TransferRequest(msg) => transfer_request(deps, env, info, msg),

        // Enclave msgs // TODO - reattach the attestations
        ExecuteMsg::Update(msg) => update(deps, env, info, msg),
        ExecuteMsg::QueryResponse(msg) => store_balance(deps, env, info, msg)
    }
}

pub mod execute {
    use cosmwasm_std::{coins, BankMsg, DepsMut, Env, Event, MessageInfo, Response};
    use cw_utils::must_pay;

    use crate::{
        error::ContractError,
        msg::execute::{QueryRequestMsg, QueryResponseMsg, Request, TransferRequestMsg, UpdateMsg},
        state::{BALANCES, DENOM, REQUESTS, STATE},
    };

    pub fn deposit(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let denom: String = DENOM.load(deps.storage)?;
        let quantity = must_pay(&info, &denom)?;

        let mut requests = REQUESTS.load(deps.storage)?;

        requests.push(Request::Deposit(info.sender, quantity));

        REQUESTS.save(deps.storage, &requests)?;

        let event = Event::new("transfer").add_attribute("action", "user");
        let resp = Response::new().add_event(event);

        Ok(resp)
    }

    pub fn withdraw(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        // TODO: verify denom

        let mut requests = REQUESTS.load(deps.storage)?;

        requests.push(Request::Withdraw(info.sender));

        REQUESTS.save(deps.storage, &requests)?;

        let event = Event::new("transfer").add_attribute("action", "user");
        let resp = Response::new().add_event(event);

        Ok(resp)
    }

    pub fn query_balance(
        _deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        _msg: QueryRequestMsg,
    ) -> Result<Response, ContractError> {
        let event = Event::new("query_balance")
            .add_attribute("query", "user")
            .add_attribute("address", info.sender);
        let resp = Response::new().add_event(event);
        Ok(resp)
    }

    pub fn transfer_request(
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: TransferRequestMsg,
    ) -> Result<Response, ContractError> {
        let mut requests = REQUESTS.load(deps.storage)?;

        requests.push(Request::Transfer(msg.ciphertext));

        REQUESTS.save(deps.storage, &requests)?;

        let event = Event::new("transfer").add_attribute("action", "user");
        let resp = Response::new().add_event(event);

        Ok(resp)
    }

    pub fn update(
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: UpdateMsg,
    ) -> Result<Response, ContractError> {
        // Store state
        STATE.save(deps.storage, &msg.ciphertext)?;

        // Clear queue
        let mut requests: Vec<Request> = REQUESTS.load(deps.storage)?;

        requests.drain(0..msg.quantity as usize);

        REQUESTS.save(deps.storage, &requests)?;

        // Process withdrawals
        let denom = DENOM.load(deps.storage)?;

        let messages = msg
            .withdrawals
            .into_iter()
            .map(|(user, funds)| BankMsg::Send {
                to_address: user.to_string(),
                amount: coins(funds.into(), &denom),
            });

        let resp = Response::new().add_messages(messages);

        Ok(resp)
    }

    pub fn store_balance(
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: QueryResponseMsg,
    ) -> Result<Response, ContractError> {
        // Store state
        BALANCES.save(
            deps.storage,
            &msg.address.to_string(),
            &msg.encrypted_bal,
        )?;

        // Emit event
        let event = Event::new("store_balance")
            .add_attribute("query", "enclave") // TODO Weird to name it enclave?
            .add_attribute(msg.address.to_string(), msg.encrypted_bal.to_string());
        let resp = Response::new().add_event(event);
        Ok(resp)
    }
}
