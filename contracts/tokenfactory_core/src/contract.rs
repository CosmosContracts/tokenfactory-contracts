#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, AllBalanceResponse, BalanceResponse, BankMsg, BankQuery, Binary, Coin, Deps,
    DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::helpers::{
    create_denom_msg, is_contract_manager, is_whitelisted, mint_factory_token_messages,
    mint_tokens_msg, pretty_denoms_output,
};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};

use token_bindings::TokenFactoryMsg;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:tokenfactory-core";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Validate existing denoms.
    let mut denoms = msg.existing_denoms.unwrap_or_default();
    for d in denoms.iter() {
        if !d.starts_with("factory/") {
            return Err(ContractError::InvalidDenom {
                denom: d.clone(),
                message: "Denom must start with 'factory/'".to_string(),
            });
        }
    }

    // Create new denoms.
    let mut new_denom_msgs: Vec<TokenFactoryMsg> = vec![];
    let mut new_mint_msgs: Vec<TokenFactoryMsg> = vec![];

    if let Some(new_denoms) = msg.new_denoms {
        if !new_denoms.is_empty() {
            for denom in new_denoms {
                let subdenom = denom.symbol.to_lowercase();
                let full_denom = format!("factory/{}/{}", env.contract.address, subdenom);

                // Add creation message.
                new_denom_msgs.push(create_denom_msg(
                    subdenom.clone(),
                    full_denom.clone(),
                    denom.clone(),
                ));

                // Add initial balance mint messages.
                if let Some(initial_balances) = denom.initial_balances {
                    if !initial_balances.is_empty() {
                        // Validate addresses.
                        for initial in initial_balances.iter() {
                            deps.api.addr_validate(&initial.address)?;
                        }

                        for b in initial_balances {
                            new_mint_msgs.push(mint_tokens_msg(
                                b.address.clone(),
                                full_denom.clone(),
                                b.amount,
                            ));
                        }
                    }
                }

                // Add to existing denoms.
                denoms.push(full_denom);
            }
        }
    }

    if denoms.is_empty() {
        return Err(ContractError::NoDenomsProvided {});
    }

    let manager = deps
        .api
        .addr_validate(&msg.manager.unwrap_or_else(|| _info.sender.to_string()))?;

    let config = Config {
        manager: manager.to_string(),
        allowed_mint_addresses: msg.allowed_mint_addresses,
        denoms,
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_messages(new_denom_msgs)
        .add_messages(new_mint_msgs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    match msg {
        // == ANYONE ==
        ExecuteMsg::Burn {} => execute_burn(deps, env, info),

        // == WHITELIST ==
        ExecuteMsg::Mint { address, denom } => execute_mint(deps, info, address, denom),

        // == MANAGER ==
        ExecuteMsg::BurnFrom { from, denom } => {
            let config = CONFIG.load(deps.storage)?;
            is_contract_manager(config, info.sender)?;

            let balance = deps.querier.query_all_balances(from.clone())?;

            let mut found = false;
            for coin in balance.iter() {
                if coin.denom == denom.denom {
                    found = true;
                }
            }

            if !found {
                return Err(ContractError::InvalidDenom {
                    denom: denom.denom,
                    message: "Denom not found in balance".to_string(),
                });
            }

            // burn from from_address
            let msg: TokenFactoryMsg = TokenFactoryMsg::BurnTokens {
                denom: denom.denom.clone(),
                amount: denom.amount,
                burn_from_address: from,
            };

            Ok(Response::new()
                .add_attribute("method", "execute_burn_from")
                .add_attribute("denom", denom.denom)
                .add_message(msg))
        }

        ExecuteMsg::TransferAdmin { denom, new_address } => {
            execute_transfer_admin(deps, info, denom, new_address)
        }

        ExecuteMsg::ForceTransfer { from, to, denom } => {
            let config = CONFIG.load(deps.storage)?;
            is_contract_manager(config, info.sender)?;

            let msg: TokenFactoryMsg = TokenFactoryMsg::ForceTransfer {
                denom: denom.denom.clone(),
                amount: denom.amount,
                from_address: from,
                to_address: to,
            };

            Ok(Response::new()
                .add_attribute("method", "execute_force_transfer")
                .add_attribute("denom", denom.denom)
                .add_message(msg))
        }

        ExecuteMsg::SetMetadata { denom, metadata } => {
            let config = CONFIG.load(deps.storage)?;
            is_contract_manager(config, info.sender)?;

            let msg: TokenFactoryMsg = TokenFactoryMsg::SetMetadata {
                denom: denom.clone(),
                metadata,
            };

            Ok(Response::new()
                .add_attribute("method", "execute_set_metadata")
                .add_attribute("denom", denom)
                .add_message(msg))
        }

        // Merge these into a modify whitelist
        ExecuteMsg::AddWhitelist { addresses } => {
            let config = CONFIG.load(deps.storage)?;
            is_contract_manager(config.clone(), info.sender)?;

            // add addresses if it is not in config.allowed_mint_addresses
            let mut updated = config.allowed_mint_addresses;
            for new in addresses {
                if !updated.contains(&new) {
                    updated.push(new);
                }
            }

            CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
                config.allowed_mint_addresses = updated;
                Ok(config)
            })?;

            Ok(Response::new().add_attribute("method", "add_whitelist"))
        }
        ExecuteMsg::RemoveWhitelist { addresses } => {
            let config = CONFIG.load(deps.storage)?;
            is_contract_manager(config.clone(), info.sender)?;

            let mut updated = config.allowed_mint_addresses;
            for remove in addresses {
                updated.retain(|a| a != &remove);
            }

            CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
                config.allowed_mint_addresses = updated;
                Ok(config)
            })?;
            Ok(Response::new().add_attribute("method", "remove_whitelist"))
        }

        ExecuteMsg::AddDenom { denoms } => {
            let config = CONFIG.load(deps.storage)?;
            is_contract_manager(config.clone(), info.sender)?;

            let mut updated_denoms = config.denoms;
            for new in denoms {
                if !updated_denoms.contains(&new) {
                    updated_denoms.push(new);
                }
            }

            CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
                config.denoms = updated_denoms;
                Ok(config)
            })?;

            Ok(Response::new().add_attribute("method", "add_denom"))
        }
        ExecuteMsg::RemoveDenom { denoms } => {
            let config = CONFIG.load(deps.storage)?;
            is_contract_manager(config.clone(), info.sender)?;

            let mut updated_denoms = config.denoms;
            for remove in denoms {
                updated_denoms.retain(|a| a != &remove);
            }

            CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
                config.denoms = updated_denoms;
                Ok(config)
            })?;
            Ok(Response::new().add_attribute("method", "remove_denom"))
        }
    }
}

pub fn execute_transfer_admin(
    deps: DepsMut,
    info: MessageInfo,
    denom: String,
    new_addr: String,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    is_contract_manager(config.clone(), info.sender)?;

    // it is possible to transfer admin in without adding to contract config. So devs need a way to reclaim admin without adding it to denoms config
    let config_denom: Option<&String> = config.denoms.iter().find(|d| d.to_string() == denom);

    if let Some(config_denom) = config_denom {
        // remove it from config
        let updated_config: Vec<String> = config
            .denoms
            .iter()
            .filter(|d| d.to_string() != *config_denom)
            .map(|d| d.to_string())
            .collect();

        CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
            config.denoms = updated_config;
            Ok(config)
        })?;
    }

    let msg = TokenFactoryMsg::ChangeAdmin {
        denom: denom.to_string(),
        new_admin_address: new_addr.to_string(),
    };

    Ok(Response::new()
        .add_attribute("method", "execute_transfer_admin")
        .add_attribute("new_admin", new_addr)
        .add_message(msg))
}

pub fn execute_mint(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
    denoms: Vec<Coin>,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    is_whitelisted(config, info.sender)?;

    let mint_msgs: Vec<TokenFactoryMsg> = mint_factory_token_messages(&address, &denoms)?;

    Ok(Response::new()
        .add_attribute("method", "execute_mint")
        .add_attribute("to_address", address)
        .add_attribute("denoms", pretty_denoms_output(&denoms))
        .add_messages(mint_msgs))
}

pub fn execute_burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    // Anyone can burn funds since they have to send them in.
    if info.funds.is_empty() {
        return Err(ContractError::InvalidFunds {});
    }

    let config = CONFIG.load(deps.storage)?;

    let (factory_denoms, send_back): (Vec<Coin>, Vec<Coin>) = info
        .funds
        .iter()
        .cloned()
        .partition(|coin| config.denoms.iter().any(|d| *d == coin.denom));

    let burn_msgs: Vec<TokenFactoryMsg> = factory_denoms
        .iter()
        .map(|coin| TokenFactoryMsg::BurnTokens {
            denom: coin.denom.clone(),
            amount: coin.amount,
            burn_from_address: env.contract.address.to_string(),
        })
        .collect();

    let bank_return_msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: send_back,
    };

    Ok(Response::new()
        .add_attribute("method", "execute_burn")
        .add_message(bank_return_msg)
        .add_messages(burn_msgs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => {
            let config = CONFIG.load(deps.storage)?;
            to_binary(&config)
        }
        QueryMsg::GetBalance { address, denom } => {
            let v = BankQuery::Balance { address, denom };
            let res: BalanceResponse = deps.querier.query(&v.into())?;
            to_binary(&res.amount)
        }

        // Since RPC's do not like to return factory/ denoms. We allow that through this query
        QueryMsg::GetAllBalances { address } => {
            let v = BankQuery::AllBalances { address };

            let res: AllBalanceResponse = deps.querier.query(&v.into())?;

            to_binary(&res.amount)
        }
    }
}
