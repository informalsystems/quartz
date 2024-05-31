use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128,
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
    state::{current_epoch_key, ObligationsItem, State, OBLIGATIONS_KEY, STATE},
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-tee-mtcs";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // must be the handled first!
    msg.0.handle_raw(deps.branch(), &env, &info)?;

    let state = State {
        owner: info.sender.to_string(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    EPOCH_COUNTER.save(deps.storage, &1)?;

    ObligationsItem::new(&current_epoch_key(OBLIGATIONS_KEY, deps.storage)?)
        .save(deps.storage, &Default::default())?;

    // store token info using cw20-base format
    let data = TokenInfo {
        name: "USD".to_string(),
        symbol: "!$".to_string(),
        decimals: 0,
        total_supply: Uint128::zero(),
        // set self as minter, so we can properly execute mint and burn
        mint: Some(MinterData {
            minter: env.contract.address.clone(),
            cap: None,
        }),
    };
    TOKEN_INFO.save(deps.storage, &data)?;

    let info = MessageInfo {
        sender: env.contract.address.clone(),
        funds: vec![],
    };

    execute_mint(
        deps.branch(),
        env.clone(),
        info.clone(),
        "wasm1qv9nel6lwtrq5jmwruxfndqw7ejskn5ysz53hp".to_owned(),
        Uint128::new(1000),
    )?;

    execute_mint(
        deps.branch(),
        env.clone(),
        info.clone(),
        "wasm1tfxrdcj5kk6rewzmmkku4d9htpjqr0kk6lcftv".to_owned(),
        Uint128::new(1000),
    )?;

    execute_mint(
        deps.branch(),
        env.clone(),
        info.clone(),
        "wasm1gjg72awjl7jvtmq4kjqp3al9p6crstpar8wgn5".to_owned(),
        Uint128::new(1000),
    )?;

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
    match msg {
        ExecuteMsg::Quartz(msg) => msg.handle_raw(deps, &env, &info).map_err(Into::into),
        ExecuteMsg::SubmitObligation(SubmitObligationMsg { ciphertext, digest }) => {
            execute::submit_obligation(deps, ciphertext, digest)
        }
        ExecuteMsg::SubmitObligations(SubmitObligationsMsg(obligations)) => {
            for o in obligations {
                execute::submit_obligation(deps.branch(), o.ciphertext, o.digest)?;
            }
            Ok(Response::new())
        }
        ExecuteMsg::SubmitSetoffs(SubmitSetoffsMsg { setoffs_enc }) => {
            execute::submit_setoffs(deps, env, info, setoffs_enc)
        }
        ExecuteMsg::InitClearing => execute::init_clearing(deps),
    }
}

pub mod execute {
    use std::collections::BTreeMap;

    use cosmwasm_std::{DepsMut, Env, HexBinary, MessageInfo, Response, StdResult};
    use cw20_base::contract::{execute_burn, execute_mint};
    use quartz_cw::state::{Hash, EPOCH_COUNTER};

    use crate::{
        state::{
            current_epoch_key, previous_epoch_key, ObligationsItem, RawHash, SetoffsItem,
            SettleOff, OBLIGATIONS_KEY, SETOFFS_KEY,
        },
        ContractError,
    };

    pub fn submit_obligation(
        deps: DepsMut,
        ciphertext: HexBinary,
        digest: HexBinary,
    ) -> Result<Response, ContractError> {
        let _: Hash = digest.to_array()?;

        // store the `(digest, ciphertext)` tuple
        ObligationsItem::new(&current_epoch_key(OBLIGATIONS_KEY, deps.storage)?).update(
            deps.storage,
            |mut obligations| {
                if let Some(_duplicate) = obligations.insert(digest.clone(), ciphertext.clone()) {
                    return Err(ContractError::DuplicateEntry);
                }
                Ok(obligations)
            },
        )?;

        Ok(Response::new()
            .add_attribute("action", "submit_obligation")
            .add_attribute("digest", digest.to_string())
            .add_attribute("ciphertext", ciphertext.to_string()))
    }

    pub fn submit_setoffs(
        mut deps: DepsMut,
        env: Env,
        _info: MessageInfo,
        setoffs_enc: BTreeMap<RawHash, SettleOff>,
    ) -> Result<Response, ContractError> {
        // store the `BTreeMap<RawHash, RawCipherText>`
        SetoffsItem::new(&previous_epoch_key(SETOFFS_KEY, deps.storage)?)
            .save(deps.storage, &setoffs_enc)?;

        for (_, so) in setoffs_enc {
            if let SettleOff::Transfer(t) = so {
                let info = MessageInfo {
                    sender: env.contract.address.clone(),
                    funds: vec![],
                };

                execute_mint(
                    deps.branch(),
                    env.clone(),
                    info.clone(),
                    t.payee.to_string(),
                    t.amount.into(),
                )?;

                let payer = deps.api.addr_validate(&t.payer.to_string())?;
                let info = MessageInfo {
                    sender: payer,
                    funds: vec![],
                };

                execute_burn(deps.branch(), env.clone(), info, t.amount.into())?;
            }
        }

        Ok(Response::new().add_attribute("action", "submit_setoffs"))
    }

    pub fn init_clearing(deps: DepsMut) -> Result<Response, ContractError> {
        EPOCH_COUNTER.update(deps.storage, |mut counter| -> StdResult<_> {
            counter += 1;
            Ok(counter)
        })?;
        Ok(Response::new().add_attribute("action", "init_clearing"))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAllSetoffs => to_json_binary(&query::get_all_setoffs(deps)?),
        QueryMsg::Balance { address } => to_json_binary(&cw20_query_balance(deps, address)?),
    }
}

pub mod query {
    use cosmwasm_std::{Deps, StdResult};

    use crate::{
        msg::GetAllSetoffsResponse,
        state::{previous_epoch_key, SetoffsItem, SETOFFS_KEY},
    };

    pub fn get_all_setoffs(deps: Deps) -> StdResult<GetAllSetoffsResponse> {
        let setoffs = SetoffsItem::new(&previous_epoch_key(SETOFFS_KEY, deps.storage)?)
            .load(deps.storage)?
            .into_iter()
            .collect();
        Ok(GetAllSetoffsResponse { setoffs })
    }
}
