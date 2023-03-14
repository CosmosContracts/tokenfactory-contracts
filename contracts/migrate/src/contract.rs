#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::Cw20ReceiveMsg;
use tokenfactory_types::msg::ExecuteMsg::Mint;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetConfig, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw20-migrate";
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
    let burn_denom = msg.burn_denom.clone();

    let cw20_token_address: Option<String> = match msg.cw20_token_address {
        Some(cw20_token_address) => {
            deps.api.addr_validate(&cw20_token_address)?;
            Some(cw20_token_address)
        }
        None => None,
    };

    if cw20_token_address.is_some() && burn_denom.is_some() {
        return Err(ContractError::InvalidDenom {
            denom: tf_denom,
            message: "Cannot set both burn_denom & cw20_token_address".to_string(),
        });
    }

    if !tf_denom.starts_with("factory/") {
        return Err(ContractError::InvalidDenom {
            denom: tf_denom,
            message: "Denom must start with 'factory/'".to_string(),
        });
    }

    // ensure contract minter address is a valid contract
    let contract_minter_address = deps
        .api
        .addr_validate(&msg.contract_minter_address)?
        .to_string();

    let state = State {
        contract_minter_address,
        cw20_token_address,
        burn_denom,
        tf_denom,
    };
    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(cw20_msg) => execute_redeem_mint(deps, info, Some(cw20_msg)),
        ExecuteMsg::Convert {} => execute_redeem_mint(deps, info, None),
    }
}

fn handle_cw20_burn(
    state: &State,
    cw20_msg: &Cw20ReceiveMsg,
    info: &MessageInfo,
) -> Result<(CosmosMsg, Uint128), ContractError> {
    let cw20_contract = info.sender.to_string();
    let amt = cw20_msg.amount;

    if cw20_contract != state.cw20_token_address.as_deref().unwrap_or_default() {
        return Err(ContractError::InvalidCW20Address {});
    }

    Ok((
        WasmMsg::Execute {
            contract_addr: cw20_contract,
            msg: to_binary(&cw20::Cw20ExecuteMsg::Burn { amount: amt })?,
            funds: vec![],
        }
        .into(),
        amt,
    ))
}

fn handle_native(state: &State, info: &MessageInfo) -> Result<(CosmosMsg, Uint128), ContractError> {
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

    if denom != state.burn_denom.as_deref().unwrap() {
        return Err(ContractError::InvalidDenom {
            denom,
            message: "This is not the correct denom to get the factory denom".to_string(),
        });
    }

    Ok((
        BankMsg::Burn {
            amount: info.funds.clone(),
        }
        .into(),
        amt,
    ))
}

pub fn execute_redeem_mint(
    deps: DepsMut,
    info: MessageInfo,
    cw20_msg: Option<Cw20ReceiveMsg>,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;

    let address_to_mint_to: String;

    let (burn_msg, amount) = if cw20_msg.is_some() {
        address_to_mint_to = cw20_msg.clone().unwrap().sender;
        handle_cw20_burn(&state, cw20_msg.as_ref().unwrap(), &info)?
    } else {
        address_to_mint_to = info.sender.to_string();
        handle_native(&state, &info)?
    };

    let mint_payload = Mint {
        address: address_to_mint_to,
        denom: vec![Coin {
            denom: state.tf_denom,
            amount,
        }],
    };

    Ok(Response::new()
        .add_attribute("method", "execute_redeem_mint")
        .add_message(burn_msg)
        .add_message(WasmMsg::Execute {
            contract_addr: state.contract_minter_address,
            msg: to_binary(&mint_payload)?,
            funds: vec![],
        }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => {
            let state = STATE.load(deps.storage)?;

            to_binary(&GetConfig {
                contract_minter_address: state.contract_minter_address,
                cw20_token_address: state.cw20_token_address,
                burn_denom: state.burn_denom,
                tf_denom: state.tf_denom,
            })
        }
    }
}
