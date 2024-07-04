use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128, Uint64,
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
        execute::{
            Cw20Transfer, FaucetMintMsg, SetLiquiditySourcesMsg, SubmitObligationMsg,
            SubmitObligationsMsg, SubmitSetoffsMsg,
        },
        ExecuteMsg, InstantiateMsg, QueryMsg,
    },
    state::{
        current_epoch_key, State, LIQUIDITY_SOURCES, LIQUIDITY_SOURCES_KEY, OBLIGATIONS, OBLIGATIONS_KEY, STATE
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
    msg.0.handle_raw(deps.branch(), &env, &info)?;

    let state = State {
        owner: info.sender.to_string(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;       
    
    
    let epoch_counter = Uint64::new(1);
    EPOCH_COUNTER.save(deps.storage, &epoch_counter)?;

    // Pre-compute the keys
    let obligations_key = current_epoch_key(OBLIGATIONS_KEY, deps.storage)?;
    let liquidity_sources_key = current_epoch_key(LIQUIDITY_SOURCES_KEY, deps.storage)?;

    // Now use the pre-computed keys
    OBLIGATIONS.save(deps.storage, &obligations_key, &Default::default())?;
    LIQUIDITY_SOURCES.save(deps.storage, &liquidity_sources_key, &Default::default())?;



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
            execute::set_liquidity_sources(deps, liquidity_sources)?;
            Ok(Response::new())
        }
        ExecuteMsg::SubmitSetoffs(attested_msg) => {
            let _ = attested_msg
                .clone()
                .handle_raw(deps.branch(), &env, &info)?;
            let SubmitSetoffsMsg { setoffs_enc } = attested_msg.msg.0;
            execute::submit_setoffs(deps, env, setoffs_enc)
        }
        ExecuteMsg::InitClearing => execute::init_clearing(deps),
        ExecuteMsg::SetLiquiditySources(SetLiquiditySourcesMsg { liquidity_sources }) => {
            execute::set_liquidity_sources(deps, liquidity_sources)
        }
    }
}

pub mod execute {
    use std::collections::BTreeMap;

    use cosmwasm_std::{DepsMut, Env, HexBinary, MessageInfo, Response, StdResult, Uint64};
    use cw20_base::contract::{execute_burn, execute_mint};
    use k256::ecdsa::VerifyingKey;
    use quartz_cw::state::{Hash, EPOCH_COUNTER};

    use crate::{
        state::{
            current_epoch_key, previous_epoch_key, RawHash,  SettleOff, LIQUIDITY_SOURCES, LIQUIDITY_SOURCES_KEY, OBLIGATIONS_KEY, SETOFFS, SETOFFS_KEY
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
    use crate::state::OBLIGATIONS;

    pub fn submit_obligation(
        deps: DepsMut,
        ciphertext: HexBinary,
        digest: HexBinary,
    ) -> Result<Response, ContractError> {
        let _: Hash = digest.to_array()?;
    
        let current_obligation_key = current_epoch_key(OBLIGATIONS_KEY, deps.storage)?;
    
        // store the `(digest, ciphertext)` tuple
        OBLIGATIONS.update(
            deps.storage,
            &current_obligation_key,
            |obligations| -> Result<_, ContractError> {
                let mut obligations = obligations.unwrap_or_default();
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

    pub fn set_liquidity_sources(
        deps: DepsMut,
        liquidity_sources: Vec<HexBinary>,
    ) -> Result<Response, ContractError> {
        // validate liquidity sources as public keys
        liquidity_sources
            .iter()
            .try_for_each(|ls| VerifyingKey::from_sec1_bytes(ls).map(|_| ()))?;
    
        let current_liquidity_key = current_epoch_key(LIQUIDITY_SOURCES_KEY, deps.storage)?;
    
        // store the liquidity sources
        let liquidity_sources_set: std::collections::BTreeSet<_> = liquidity_sources.into_iter().collect();
        LIQUIDITY_SOURCES.save(deps.storage, &current_liquidity_key, &liquidity_sources_set)?;
    
        Ok(Response::default())
    }
   
    pub fn submit_setoffs(
        mut deps: DepsMut,
        env: Env,
        setoffs_enc: BTreeMap<RawHash, SettleOff>,
    ) -> Result<Response, ContractError> {
        // store the `BTreeMap<RawHash, RawCipherText>`
        let previous_epoch_key = previous_epoch_key(SETOFFS_KEY, deps.storage)?;
        SETOFFS.save(deps.storage, &previous_epoch_key, &setoffs_enc)?;
    
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
            counter = counter.saturating_add(Uint64::from(1u64));
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
    use cosmwasm_std::{Deps, StdResult, Uint64};

    use crate::{
        msg::{GetAllSetoffsResponse, GetLiquiditySourcesResponse},
        state::{
            current_epoch_key, epoch_key, previous_epoch_key, LIQUIDITY_SOURCES, SETOFFS,
            LIQUIDITY_SOURCES_KEY, SETOFFS_KEY,
        },
    };

    pub fn get_all_setoffs(deps: Deps) -> StdResult<GetAllSetoffsResponse> {
        let previous_epoch_key = previous_epoch_key(SETOFFS_KEY, deps.storage)?;
        let setoffs_map = SETOFFS.load(deps.storage, &previous_epoch_key)?;
        let setoffs = setoffs_map.into_iter().collect();
        Ok(GetAllSetoffsResponse { setoffs })
    }

    pub fn get_liquidity_sources(
        deps: Deps,
        epoch: Option<Uint64>,
    ) -> StdResult<GetLiquiditySourcesResponse> {
        let epoch_key = match epoch {
            None => current_epoch_key(LIQUIDITY_SOURCES_KEY, deps.storage)?,
            Some(e) => epoch_key(LIQUIDITY_SOURCES_KEY, e)?,
        };
    
        let liquidity_sources_set = LIQUIDITY_SOURCES.load(deps.storage, &epoch_key)?;
        let liquidity_sources = liquidity_sources_set.into_iter().collect();
        Ok(GetLiquiditySourcesResponse { liquidity_sources })
    }
}