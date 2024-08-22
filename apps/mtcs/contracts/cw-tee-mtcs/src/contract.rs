use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint64,
};
use cw2::set_contract_version;
use cw20_base::contract::query_balance as cw20_query_balance;
use quartz_common::contract::{handler::RawHandler, state::EPOCH_COUNTER};

use crate::{
    error::ContractError,
    msg::{
        execute::{
            Cw20Transfer, FaucetMintMsg, SetLiquiditySourcesMsg, SubmitObligationMsg,
            SubmitObligationsMsg, SubmitSetoffsMsg,
        },
        ExecuteMsg, InstantiateMsg, QueryMsg,
    },
    state::{
        current_epoch_key, ObligationsItem, State, LIQUIDITY_SOURCES, LIQUIDITY_SOURCES_KEY,
        OBLIGATIONS_KEY, STATE,
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
) -> Result<Response, ContractError> {
    // must be the handled first!
    msg.quartz.handle_raw(deps.branch(), &env, &info)?;

    let state = State {
        owner: info.sender.to_string(),
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    let epoch_counter = Uint64::new(1);
    EPOCH_COUNTER.save(deps.storage, &epoch_counter)?;

    ObligationsItem::new_dyn(current_epoch_key(OBLIGATIONS_KEY, deps.storage)?)
        .save(deps.storage, &Default::default())?;

    // TODO: this can be removed. We don't need to instantiate liquidity sources, users will do so when submitting obligations
    let epoch = current_epoch_key(LIQUIDITY_SOURCES_KEY, deps.storage)?;
    LIQUIDITY_SOURCES.save(deps.storage, &epoch, &vec![])?;

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
        ExecuteMsg::FaucetMint(FaucetMintMsg { recipient, amount }) => {
            execute::faucet_mint(deps, env, recipient, amount)
        }
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
        ExecuteMsg::SubmitSetoffs(attested_msg) => {
            // let _ = attested_msg
            //     .clone()
            //     .handle_raw(deps.branch(), &env, &info)?;
            let SubmitSetoffsMsg { setoffs_enc } = attested_msg.msg.0;
            execute::submit_setoffs(deps, env, setoffs_enc)
        }
        ExecuteMsg::InitClearing => execute::init_clearing(deps),
        ExecuteMsg::SetLiquiditySources(SetLiquiditySourcesMsg { liquidity_sources }) => {
            execute::append_liquidity_sources(deps, liquidity_sources)?;
            Ok(Response::new())
        }
    }
}

pub mod execute {
    use std::collections::BTreeMap;

    use cosmwasm_std::{
        to_json_binary, Addr, DepsMut, Env, HexBinary, MessageInfo, Response, StdResult, Storage,
        SubMsg, Uint64, WasmMsg,
    };
    use cw20_base::contract::execute_mint;
    use quartz_common::contract::state::{Hash, EPOCH_COUNTER};

    // use mtcs_overdraft::msg::ExecuteMsg as OverdraftExecuteMsg;
    use crate::msg::OverdraftExecuteMsg; // TODO: change when dependency issue fiexed
    use crate::{
        msg::execute::EscrowExecuteMsg,
        state::{
            current_epoch_key, previous_epoch_key, LiquiditySource, LiquiditySourceType,
            ObligationsItem, RawHash, SetoffsItem, SettleOff, Transfer, LIQUIDITY_SOURCES,
            LIQUIDITY_SOURCES_KEY, OBLIGATIONS_KEY, SETOFFS_KEY,
        },
        ContractError,
    };

    pub fn faucet_mint(
        mut deps: DepsMut,
        env: Env,
        recipient: String,
        amount: u64,
    ) -> Result<Response, ContractError> {
        let info = MessageInfo {
            sender: env.contract.address.clone(),
            funds: vec![],
        };

        execute_mint(
            deps.branch(),
            env.clone(),
            info.clone(),
            recipient.to_string(),
            amount.into(),
        )?;

        Ok(Response::new().add_attribute("action", "faucet_mint"))
    }

    pub fn submit_obligation(
        deps: DepsMut,
        ciphertext: HexBinary,
        digest: HexBinary,
    ) -> Result<Response, ContractError> {
        let _: Hash = digest.to_array()?;

        // store the `(digest, ciphertext)` tuple
        let obligs_key =
            ObligationsItem::new_dyn(current_epoch_key(OBLIGATIONS_KEY, deps.storage)?);

        let mut epoch_obligation = obligs_key.may_load(deps.storage)?.unwrap_or_default();

        if let Some(_duplicate) = epoch_obligation.insert(digest.clone(), ciphertext.clone()) {
            return Err(ContractError::DuplicateEntry);
        }

        obligs_key.save(deps.storage, &epoch_obligation)?;

        Ok(Response::new()
            .add_attribute("action", "submit_obligation")
            .add_attribute("digest", digest.to_string())
            .add_attribute("ciphertext", ciphertext.to_string()))
    }

    pub fn append_liquidity_sources(
        deps: DepsMut,
        new_liquidity_sources: Vec<LiquiditySource>,
    ) -> Result<(), ContractError> {
        let epoch = current_epoch_key(LIQUIDITY_SOURCES_KEY, deps.storage)?;
        let mut liquidity_sources = LIQUIDITY_SOURCES
            .may_load(deps.storage, &epoch)?
            .unwrap_or_default();

        let mut new_sources = vec![];
        for liquidity_source in new_liquidity_sources {
            // Validate the Cosmos address
            let address = deps.api.addr_validate(liquidity_source.address.as_ref())?;

            let liquidity_source = LiquiditySource {
                address: address.clone(),
                source_type: liquidity_source.source_type,
            };

            new_sources.push(liquidity_source);
        }

        liquidity_sources.append(&mut new_sources);

        // Save the new liquidity sources
        LIQUIDITY_SOURCES.save(deps.storage, &epoch, &liquidity_sources)?;

        Ok(())
    }

    pub fn submit_setoffs(
        deps: DepsMut,
        _env: Env,
        setoffs_enc: BTreeMap<RawHash, SettleOff>,
    ) -> Result<Response, ContractError> {
        // Store the setoffs
        SetoffsItem::new_dyn(previous_epoch_key(SETOFFS_KEY, deps.storage)?)
            .save(deps.storage, &setoffs_enc)?;

        let mut messages = vec![];

        for (_, so) in setoffs_enc {
            if let SettleOff::Transfer(t) = so {
                // Check if either payer or payee is a liquidity source
                let payer_source = find_liquidity_source(deps.storage, &t.payer)?;
                let payee_source = find_liquidity_source(deps.storage, &t.payee)?;

                match (payer_source, payee_source) {
                    (Some(source), None) => {
                        // Payer is a liquidity source
                        let msg = create_transfer_message(&source, &t, true)?;
                        messages.push(msg);
                    }
                    (None, Some(source)) => {
                        // Payee is a liquidity source
                        let msg = create_transfer_message(&source, &t, false)?;
                        messages.push(msg);
                    }
                    (_, _) => {
                        // As of now, transfers should only be between a user and liquidity source.
                        return Err(ContractError::LiquiditySourceNotFound {});
                    }
                }
            }
        }

        Ok(Response::new()
            .add_submessages(messages)
            .add_attribute("action", "submit_setoffs"))
    }

    fn find_liquidity_source(
        storage: &dyn Storage,
        address: &Addr,
    ) -> Result<Option<LiquiditySource>, ContractError> {
        // TODO: check that .ok() is correct here
        let liquidity_sources = LIQUIDITY_SOURCES.load(
            storage,
            &previous_epoch_key(LIQUIDITY_SOURCES_KEY, storage)?,
        )?;

        Ok(liquidity_sources
            .into_iter()
            .find(|lqs| lqs.address == address))
    }

    fn create_transfer_message(
        source: &LiquiditySource,
        transfer: &Transfer,
        is_payer: bool,
    ) -> Result<SubMsg, ContractError> {
        let msg = match source.source_type {
            LiquiditySourceType::Escrow => {
                let (payer, payee, amount) = if is_payer {
                    (
                        transfer.payer.to_string(),
                        transfer.payee.to_string(),
                        vec![transfer.amount.clone()],
                    )
                } else {
                    // If the liquidity source is the payee, we swap payer and payee
                    (
                        transfer.payee.to_string(),
                        transfer.payer.to_string(),
                        vec![transfer.amount.clone()],
                    )
                };

                WasmMsg::Execute {
                    contract_addr: source.address.to_string(),
                    msg: to_json_binary(&EscrowExecuteMsg::ExecuteSetoff {
                        payer,
                        payee,
                        amount,
                    })?,
                    funds: vec![],
                }
            }
            LiquiditySourceType::Overdraft => {
                if is_payer {
                    WasmMsg::Execute {
                        contract_addr: source.address.to_string(),
                        msg: to_json_binary(&OverdraftExecuteMsg::IncreaseBalance {
                            receiver: transfer.payee.clone(),
                            amount: transfer.amount.1,
                        })?,
                        funds: vec![],
                    }
                } else {
                    WasmMsg::Execute {
                        contract_addr: source.address.to_string(),
                        msg: to_json_binary(&OverdraftExecuteMsg::DecreaseBalance {
                            receiver: transfer.payer.clone(),
                            amount: transfer.amount.1,
                        })?,
                        funds: vec![],
                    }
                }
            }
            LiquiditySourceType::External => {
                return Err(ContractError::UnsupportedLiquiditySource {})
            }
        };

        Ok(SubMsg::new(msg))
    }

    pub fn init_clearing(deps: DepsMut) -> Result<Response, ContractError> {
        EPOCH_COUNTER.update(deps.storage, |mut counter| -> StdResult<_> {
            counter = counter.checked_add(Uint64::new(1))?;
            Ok(counter)
        })?;

        // Initializing data for next Epoch
        let liquidity_epoch_key = current_epoch_key(LIQUIDITY_SOURCES_KEY, deps.storage)?;

        ObligationsItem::new_dyn(current_epoch_key(OBLIGATIONS_KEY, deps.storage)?)
            .save(deps.storage, &Default::default())?;
        LIQUIDITY_SOURCES.save(deps.storage, &liquidity_epoch_key, &vec![])?;

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
    use cosmwasm_std::{Deps, StdResult, Uint64};

    use crate::{
        msg::{GetAllSetoffsResponse, GetLiquiditySourcesResponse},
        state::{
            current_epoch_key, epoch_key, previous_epoch_key, SetoffsItem, LIQUIDITY_SOURCES,
            LIQUIDITY_SOURCES_KEY, SETOFFS_KEY,
        },
    };

    pub fn get_all_setoffs(deps: Deps) -> StdResult<GetAllSetoffsResponse> {
        let setoffs = SetoffsItem::new_dyn(previous_epoch_key(SETOFFS_KEY, deps.storage)?)
            .load(deps.storage)?
            .into_iter()
            .collect();
        Ok(GetAllSetoffsResponse { setoffs })
    }

    // Function to get liquidity sources for a specific epoch
    pub fn get_liquidity_sources(
        deps: Deps,
        epoch: Option<Uint64>,
    ) -> StdResult<GetLiquiditySourcesResponse> {
        let epoch_key = match epoch {
            None => current_epoch_key(LIQUIDITY_SOURCES_KEY, deps.storage)?,
            Some(e) => epoch_key(LIQUIDITY_SOURCES_KEY, e)?,
        };

        let liquidity_sources = LIQUIDITY_SOURCES.load(deps.storage, &epoch_key)?;

        Ok(GetLiquiditySourcesResponse { liquidity_sources })
    }
}
