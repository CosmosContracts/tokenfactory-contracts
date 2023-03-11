use std::ops::Add;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{
    to_binary, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, SubMsg, Uint128, WasmMsg, CosmosMsg,
};

use cw2::set_contract_version;
use cw20::{    
    DownloadLogoResponse, EmbeddedLogo, Logo, MarketingInfoResponse, MinterResponse,
    TokenInfoResponse,
};
use cw_utils::ensure_from_older_version;

use crate::enumerable::{query_all_accounts};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{BALANCES, LOGO, MARKETING_INFO,
    TOKEN_INFO, ALLOWANCES, ALLOWANCES_SPENDER, MIGRATION, MigrateConfig,
};

use tokenfactory_types::msg::ExecuteMsg::Mint;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw20-base";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Err(ContractError::CannotInstantiate {})
}

#[cfg_attr(not(feature = "library"), entry_point)]
#[allow(unused_variables)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::MigrateTokens { limit } => {            
            // add from minter check here? or permissionless (for bots to do)?

            let migrate_config = MIGRATION.load(deps.storage)?;
        
            let accs = query_all_accounts(deps.as_ref(), migrate_config.start_after_addr, limit)?;
            
            let mut mint_msgs: Vec<CosmosMsg> = vec![];
            let mut amount_change: Uint128 = Uint128::zero();

            // iterate all accounts, and mint 1:1 tokens
            for account_addr_string in accs.accounts.clone() {  
                let acc_addr = &deps.api.addr_validate(&account_addr_string)?;                 
                let balance = BALANCES.load(deps.storage, acc_addr)?;                

                amount_change = amount_change.add(balance.clone());

                let mint_msg = Mint {
                    address: account_addr_string,
                    denom: vec![Coin {
                        amount: balance.clone(),
                        denom: migrate_config.tf_denom.clone(),
                    }],
                };

                let wasm_mint = WasmMsg::Execute {
                    contract_addr: migrate_config.tf_core_address.clone(),
                    msg: to_binary(&mint_msg).unwrap(),
                    funds: vec![],
                };     


                if migrate_config.burn_cw20_balances {                    
                    BALANCES.update(
                        deps.storage,
                        acc_addr,
                        |balance: Option<Uint128>| -> StdResult<_> {                            
                            Ok(Uint128::zero())
                        },
                    )?;
                }                           

                mint_msgs.push(CosmosMsg::Wasm(wasm_mint));
            }

            // update the last address we migrated
            let last_addr = accs.accounts.last().unwrap().clone();

            MIGRATION.update(deps.storage, |mut config| -> StdResult<_> {
                config.start_after_addr = Some(last_addr.clone());
                Ok(config)
            })?;

            TOKEN_INFO.update(deps.storage, |mut info| -> StdResult<_> {
                info.total_supply = info.total_supply.checked_sub(amount_change)?;
                Ok(info)
            })?;            

            Ok(Response::new().add_messages(mint_msgs).add_attribute("last_account", last_addr.to_string()))
        },
    }
}


#[cfg_attr(not(feature = "library"), entry_point)]
#[allow(unused_variables)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Minter {} => to_binary(&None as &Option<MinterResponse>),
    
        // Kept the same
        QueryMsg::MarketingInfo {} => to_binary(&MARKETING_INFO.may_load(deps.storage)?.unwrap_or_default()),
        QueryMsg::DownloadLogo {} => to_binary(&query_download_logo(deps)?),
        QueryMsg::TokenInfo {} => to_binary(&query_token_info(deps)?),        
        QueryMsg::AllAccounts { start_after, limit } => {
            to_binary(&query_all_accounts(deps, start_after, limit)?)
        }
    }
}

pub fn query_token_info(deps: Deps) -> StdResult<TokenInfoResponse> {
    let info = TOKEN_INFO.load(deps.storage)?;
    let res = TokenInfoResponse {
        name: info.name,
        symbol: info.symbol,
        decimals: info.decimals,
        total_supply: info.total_supply,
    };
    Ok(res)
}

pub fn query_download_logo(deps: Deps) -> StdResult<DownloadLogoResponse> {
    let logo = LOGO.load(deps.storage)?;
    match logo {
        Logo::Embedded(EmbeddedLogo::Svg(logo)) => Ok(DownloadLogoResponse {
            mime_type: "image/svg+xml".to_owned(),
            data: logo,
        }),
        Logo::Embedded(EmbeddedLogo::Png(logo)) => Ok(DownloadLogoResponse {
            mime_type: "image/png".to_owned(),
            data: logo,
        }),
        Logo::Url(_) => Err(StdError::not_found("logo")),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    ensure_from_older_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;    

    // We migrate, then mint tokens to the other contract.
    deps.api.addr_validate(&msg.tf_core_address)?;

    // check if msg.tf_denom starts with factory
    if !msg.tf_denom.starts_with("factory") {
        return Err(ContractError::InvalidDenom {});
    }

    // Save ther data for who mints & the denom to mint from said contract
    // This contract addr has to be whitelisted to do this.
    MIGRATION.save(deps.storage, &MigrateConfig {
        tf_core_address: msg.tf_core_address,     
        tf_denom: msg.tf_denom,  
        start_after_addr: None,
        burn_cw20_balances: msg.burn_cw20_balances,
    })?;

        
    // clear all allowances that will never be used again
    ALLOWANCES.clear(deps.storage);
    ALLOWANCES_SPENDER.clear(deps.storage);
    
    Ok(Response::default())
}