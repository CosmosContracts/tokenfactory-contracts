#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, WasmMsg,
};
use cw2::set_contract_version;
use tokenfactory_types::msg::ExecuteMsg::Mint;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetConfig, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};

const CONTRACT_NAME: &str = "crates.io:ibc-migrate";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let tf_denom = msg.tf_denom.clone();
    if !tf_denom.starts_with("factory/") {
        return Err(ContractError::InvalidDenom {
            denom: tf_denom,
            message: "Denom must start with 'factory/'".to_string(),
        });
    }

    let contract_minter_address = deps.api.addr_validate(&msg.contract_minter_address)?;

    let state = State {
        contract_minter_address,
        burn_denom: msg.burn_denom,
        tf_denom,
    };
    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Convert {} => execute_convert(deps, info, env),
    }
}

pub fn execute_convert(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;

    if info.funds.is_empty() {
        return Err(ContractError::NoFundsSent {});
    }

    if info.funds.len() > 1 {
        return Err(ContractError::InvalidDenom {
            denom: info.funds.iter().map(|c| c.denom.clone()).collect(),
            message: "Only one denom can be sent".to_string(),
        });
    }

    let denom = info.funds[0].denom.clone();
    let amt = info.funds[0].amount;

    if denom != state.burn_denom {
        return Err(ContractError::InvalidDenom {
            denom: denom,
            message: "This is not the correct denom to get the factory denom".to_string(),
        });
    }

    let mut messages: Vec<CosmosMsg> = vec![BankMsg::Burn { amount: info.funds }.into()];

    let mint_payload = Mint {
        address: info.sender.to_string(),
        denom: vec![Coin {
            denom: state.tf_denom,
            amount: amt,
        }],
    };

    messages.push(
        WasmMsg::Execute {
            contract_addr: state.contract_minter_address.to_string(),
            msg: to_binary(&mint_payload)?,
            funds: vec![],
        }
        .into(),
    );

    Ok(Response::new()
        .add_attribute("method", "execute_convert")
        .add_messages(messages))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => {
            let state: State = STATE.load(deps.storage)?;
            let contract_minter_address = state.contract_minter_address.to_string();

            to_binary(&GetConfig {
                contract_minter_address,
                burn_denom: state.burn_denom,
                tf_denom: state.tf_denom,
            })
        }
    }
}
