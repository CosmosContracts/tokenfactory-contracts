#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    WasmMsg,
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
    let cw20_token_address = deps.api.addr_validate(&msg.cw20_token_address)?;

    if !tf_denom.starts_with("factory/") {
        return Err(ContractError::InvalidDenom {
            denom: tf_denom,
            message: "Denom must start with 'factory/'".to_string(),
        });
    }

    deps.api.addr_validate(&msg.contract_minter_address)?;

    let state = State {
        contract_minter_address: msg.contract_minter_address,
        cw20_token_address: cw20_token_address.to_string(),
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
        ExecuteMsg::Receive(cw20_msg) => execute_redeem_mint(deps, info, env, cw20_msg),
    }
}

pub fn execute_redeem_mint(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let cw20_contract = info.sender.to_string();
    let state = STATE.load(deps.storage)?;

    if cw20_contract != state.cw20_token_address {
        return Err(ContractError::InvalidCW20Address {});
    }

    let mut messages: Vec<CosmosMsg> = vec![];

    // Burn the CW20 since it is in our possession now
    let cw20_burn = cw20::Cw20ExecuteMsg::Burn {
        amount: cw20_msg.amount,
    };
    messages.push(
        WasmMsg::Execute {
            contract_addr: cw20_contract,
            msg: to_binary(&cw20_burn)?,
            funds: vec![],
        }
        .into(),
    );

    // Mint the token
    let mint_payload = Mint {
        address: cw20_msg.sender.clone(),
        denom: vec![Coin {
            denom: state.tf_denom,
            amount: cw20_msg.amount,
        }],
    };
    let wasm_mint_msg = WasmMsg::Execute {
        contract_addr: state.contract_minter_address,
        msg: to_binary(&mint_payload)?,
        funds: vec![],
    };

    messages.push(wasm_mint_msg.into());

    Ok(Response::new()
        .add_attribute("method", "redeem_tokens")
        .add_messages(messages))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => {
            let state = STATE.load(deps.storage)?;

            to_binary(&GetConfig {
                contract_minter_address: state.contract_minter_address,
                cw20_token_address: state.cw20_token_address,
                tf_denom: state.tf_denom,
            })
        }
    }
}
