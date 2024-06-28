use std::collections::BTreeSet;

use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128,
};
use cw2::set_contract_version;
use cw20_base::{
    contract::query_balance as cw20_query_balance,
    state::{MinterData, TokenInfo, TOKEN_INFO},
};
use quartz_cw::{handler::RawHandler, state::EPOCH_COUNTER};

use crate::{
    error::ContractError,
    msg::{
        execute::{Cw20Transfer, SubmitObligationMsg, SubmitObligationsMsg, SubmitSetoffsMsg},
        ExecuteMsg, InstantiateMsg, QueryMsg,
    },
    state::{
        current_epoch_key,  LiquiditySource, LiquiditySourceType, LiquiditySourcesItem,
        ObligationsItem, State, LIQUIDITY_SOURCES, LIQUIDITY_SOURCES_KEY, OBLIGATIONS_KEY, STATE,
    },
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
    escrow_address: String,
    overdraft_address: String,
) -> Result<Response, ContractError> {
    // must be the handled first!
    msg.0.handle_raw(deps.branch(), &env, &info)?;

    let state = State {
        owner: info.sender.to_string(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    ObligationsItem::new(&current_epoch_key(OBLIGATIONS_KEY, deps.storage)?)
        .save(deps.storage, &Default::default())?;

    // set escrow contract address
    let escrow = LiquiditySource {
        address: deps.api.addr_validate(&escrow_address)?,
        source_type: LiquiditySourceType::Escrow,
    };

    // set overdraft contract address
    let overdraft = LiquiditySource {
        address: deps.api.addr_validate(&overdraft_address)?,
        source_type: LiquiditySourceType::Overdraft,
    };

    LIQUIDITY_SOURCES.save(deps.storage, "1", &escrow)?;
    LIQUIDITY_SOURCES.save(deps.storage, "1", &overdraft)?;

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
        ExecuteMsg::Transfer(Cw20Transfer { recipient, amount }) => Ok(
            cw20_base::contract::execute_transfer(deps, env, info, recipient, amount.into())?,
        ),
        ExecuteMsg::SubmitObligation(SubmitObligationMsg { ciphertext, digest }) => {
            execute::submit_obligation(deps, ciphertext, digest)
        }
        ExecuteMsg::SubmitObligations(SubmitObligationsMsg {
            obligations,
            liquidity_sources,
        }) => {
            for o in obligations {
                execute::submit_obligation(deps.branch(), o.ciphertext, o.digest)?;
            }
            execute::append_liquidity_sources(deps, liquidity_sources)?;
            Ok(Response::new())
        }
        ExecuteMsg::SubmitSetoffs(SubmitSetoffsMsg { setoffs_enc }) => {
            execute::submit_setoffs(deps, env, setoffs_enc)
        }
        ExecuteMsg::InitClearing => execute::init_clearing(deps),
    }
}

pub mod execute {
    use std::{collections::BTreeMap, ops::DerefMut};

    use cosmwasm_std::{Addr, DepsMut, Env, HexBinary, MessageInfo, Response, StdResult};
    use cw20_base::contract::{execute_burn, execute_mint};
    use quartz_cw::state::{Hash, EPOCH_COUNTER};

    use crate::{
        state::{
            current_epoch_key, previous_epoch_key, LiquiditySource, LiquiditySourceType,
            ObligationsItem, RawHash, SetoffsItem, SettleOff,
            LIQUIDITY_SOURCES, LIQUIDITY_SOURCES_KEY, OBLIGATIONS_KEY,
            SETOFFS_KEY,
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

    pub fn append_liquidity_sources(
        deps: DepsMut,
        liquidity_sources: Vec<LiquiditySource>,
    ) -> Result<(), ContractError> {
        let epoch = current_epoch_key(LIQUIDITY_SOURCES_KEY, deps.storage)?;

        for liquidity_source in liquidity_sources {
            // Validate the Cosmos address
            let address = deps.api.addr_validate(&liquidity_source.address.to_string())?;

            let liquidity_source = LiquiditySource {
                address: address.clone(),
                source_type: liquidity_source.source_type
            };

            // Save the new liquidity source
            LIQUIDITY_SOURCES.save(deps.storage, &epoch, &liquidity_source)?;
        }

        Ok(())
    }

    pub fn submit_setoffs(
        mut deps: DepsMut,
        env: Env,
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
        QueryMsg::GetLiquiditySources { epoch } => {
            to_json_binary(&query::get_liquidity_sources(deps, epoch)?)
        }
        QueryMsg::Balance { address } => to_json_binary(&cw20_query_balance(deps, address)?),
    }
}

pub mod query {
    use cosmwasm_std::{Deps, Order, StdResult};

    use crate::{
        msg::{GetAllSetoffsResponse, GetLiquiditySourcesResponse},
        state::{
            current_epoch_key, epoch_key, previous_epoch_key, LiquiditySource, SetoffsItem,
            LIQUIDITY_SOURCES, LIQUIDITY_SOURCES_KEY, SETOFFS_KEY,
        }
      };

    pub fn get_all_setoffs(deps: Deps) -> StdResult<GetAllSetoffsResponse> {
        let setoffs = SetoffsItem::new(&previous_epoch_key(SETOFFS_KEY, deps.storage)?)
            .load(deps.storage)?
            .into_iter()
            .collect();
        Ok(GetAllSetoffsResponse { setoffs })
    }

     // Function to get liquidity sources for a specific epoch
    pub fn get_liquidity_sources(
        deps: Deps,
        epoch: Option<usize>,
    ) -> StdResult<GetLiquiditySourcesResponse> {
        let epoch_key = match epoch {
            None => current_epoch_key(LIQUIDITY_SOURCES_KEY, deps.storage)?,
            Some(e) => epoch_key(LIQUIDITY_SOURCES_KEY, e)?,
        };

        let liquidity_sources: Vec<LiquiditySource> = LIQUIDITY_SOURCES
            .range(deps.storage, None, None, Order::Ascending)
            .filter_map(|result| {
                result.ok().and_then(|(key, value)| {
                    if key.starts_with(&epoch_key) {
                        Some(value)
                    } else {
                        None
                    }
                })
            })
            .collect();

        Ok(GetLiquiditySourcesResponse { liquidity_sources })
    }
}
