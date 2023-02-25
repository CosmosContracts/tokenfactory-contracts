#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, WasmMsg,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, STATE};

use tokenfactory_types::msg::ExecuteMsg::Mint;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // if msg.core_factory_address is some

    let config;
    if msg.core_factory_address.is_some() {
        let core_addr = deps.api.addr_validate(&msg.core_factory_address.unwrap())?;
        config = Config {
            core_address: Some(core_addr.to_string()),
        };
    } else {
        config = Config { core_address: None };
    }

    STATE.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        // While this is named MintTokens, think of it as any other message which does something
        ExecuteMsg::MintTokens {
            core_factory_address,
            denoms,
            to_address,
        } => {
            let core_tf_addr: Addr;
            if let Some(addr) = core_factory_address {
                core_tf_addr = deps.api.addr_validate(&addr)?;
            } else {
                core_tf_addr = deps
                    .api
                    .addr_validate(&STATE.load(deps.storage)?.core_address.unwrap())?;
            }

            let payload = Mint {
                address: to_address,
                denom: denoms,
            };
            let wasm_msg = WasmMsg::Execute {
                contract_addr: core_tf_addr.to_string(),
                msg: to_binary(&payload)?,
                funds: vec![],
            };

            Ok(Response::new()
                .add_attribute("method", "execute_mint_tokens")
                .add_message(wasm_msg))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => {
            let state = STATE.load(deps.storage)?;
            to_binary(&state)
        }
    }
}
