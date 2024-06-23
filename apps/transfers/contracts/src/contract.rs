use cosmwasm_std::{entry_point, DepsMut, Env, HexBinary, MessageInfo, Response};
use quartz_cw::handler::RawHandler;

use crate::{
    error::ContractError,
    msg::{
        execute::{Request, UpdateMsg},
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
        ExecuteMsg::Quartz(msg) => msg.handle_raw(deps, &env, &info).map_err(Into::into),
        ExecuteMsg::TransferRequest(msg) => transfer_request(deps, env, info, msg),
        ExecuteMsg::Update(attested_msg) => {
            let _ = attested_msg
                .clone()
                .handle_raw(deps.branch(), &env, &info)?;

            // Extract underlying UpdateMsg and pass to update()
            update(deps, env, info, UpdateMsg(attested_msg.msg))
        }
        ExecuteMsg::Deposit => deposit(deps, env, info),
        ExecuteMsg::Withdraw => withdraw(deps, env, info),
        ExecuteMsg::ClearTextTransferRequest(_) => unimplemented!(),
    }
}

pub mod execute {
    use cosmwasm_std::{coins, BankMsg, DepsMut, Env, Event, MessageInfo, Response};
    use cw_utils::must_pay;

    use crate::{
        error::ContractError,
        msg::execute::{Request, TransferRequestMsg, UpdateMsg},
        state::{DENOM, REQUESTS, STATE},
    };

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
        STATE.save(deps.storage, &msg.0.ciphertext)?;

        // Clear queue
        let mut requests: Vec<Request> = REQUESTS.load(deps.storage)?;

        requests.drain(0..msg.0.quantity as usize);

        REQUESTS.save(deps.storage, &requests)?;

        // Process withdrawals
        let denom = DENOM.load(deps.storage)?;

        let messages = msg
            .0
            .withdrawals
            .into_iter()
            .map(|(user, funds)| BankMsg::Send {
                to_address: user.to_string(),
                amount: coins(funds.into(), &denom),
            });

        let resp = Response::new().add_messages(messages);

        Ok(resp)
    }

    pub fn deposit(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let denom = DENOM.load(deps.storage)?;
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
}
