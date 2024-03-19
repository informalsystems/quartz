use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use quartz_cw::handler::RawHandler;

use crate::error::ContractError;
use crate::msg::execute::{SubmitObligationMsg, SubmitSetoffsMsg};
use crate::msg::QueryMsg;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{current_epoch_key, ObligationsItem, State, OBLIGATIONS_KEY, STATE};

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

    ObligationsItem::new(&current_epoch_key(OBLIGATIONS_KEY, deps.storage)?)
        .save(deps.storage, &Default::default())?;

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
        ExecuteMsg::Quartz(msg) => msg.handle_raw(deps, &env, &info).map_err(Into::into),
        ExecuteMsg::SubmitObligation(SubmitObligationMsg { ciphertext, digest }) => {
            execute::submit_obligation(deps, ciphertext, digest)
        }
        ExecuteMsg::SubmitSetoffs(SubmitSetoffsMsg { setoffs_enc }) => {
            execute::submit_setoffs(deps, setoffs_enc)
        }
    }
}

pub mod execute {
    use std::collections::BTreeMap;

    use cosmwasm_std::{DepsMut, HexBinary, Response};
    use quartz_cw::state::Hash;

    use crate::state::{
        current_epoch_key, ObligationsItem, RawCipherText, RawHash, SetoffsItem, OBLIGATIONS_KEY,
        SETOFFS_KEY,
    };
    use crate::ContractError;

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
        deps: DepsMut,
        setoffs_enc: BTreeMap<RawHash, RawCipherText>,
    ) -> Result<Response, ContractError> {
        // store the `BTreeMap<RawHash, RawCipherText>`
        SetoffsItem::new(&current_epoch_key(SETOFFS_KEY, deps.storage)?)
            .save(deps.storage, &setoffs_enc)?;

        Ok(Response::new().add_attribute("action", "submit_setoffs"))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {}
}
