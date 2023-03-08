#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{
    to_binary, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, SubMsg, Uint128, WasmMsg,
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
            // Maybe allow anyone to do this to migrate tokens permissionlessly / with bots. Save the lasrt key in the limit to state so we know where to start back?
            // This way we do not have to burn tokens incase something goes very wrong.



                // iterate all accounts, and mint 1:1 tokens
            //     let data = BALANCES
            //     .range(deps.storage, None, None, Ascending)
            //     .collect::<StdResult<Vec<_>>>()?;   
            // let mut coin = vec![Coin {
            //     amount: Uint128::zero(),
            //     denom: msg.tf_denom,
            // }];

            // let mint_msgs = data
            //     .iter()
            //     .map(|(addr, balance)| {
            //         coin[0].amount = balance.clone();
            //         let mint_msg = Mint {
            //             address: addr.to_string(),
            //             denom: coin.clone(),
            //         };

            //         let wasm_mint = WasmMsg::Execute {
            //             contract_addr: tf_core.clone(),
            //             msg: to_binary(&mint_msg).unwrap(),
            //             funds: vec![],
            //         };

            //         SubMsg::new(wasm_mint)
            //     })
            //     .collect::<Vec<_>>();
            // .add_submessages(mint_msgs) 

            // for each account, burn their tokens and mint new ones through tf_core_addr
            // if too many, maybe we have to set this contract as the admin and mint tokens directly through TokenMsg here?
            // Then transfer back to user / contract who called the migrate message on this or something


            // lower balance
            // BALANCES.update(
            //     deps.storage,
            //     &info.sender,
            //     |balance: Option<Uint128>| -> StdResult<_> {
            //         Ok(balance.unwrap_or_default().checked_sub(amount)?)
            //     },
            // )?;
            // reduce total_supply
            // TOKEN_INFO.update(deps.storage, |mut info| -> StdResult<_> {
            //     info.total_supply = info.total_supply.checked_sub(amount)?;
            //     Ok(info)
            // })?;
            // let res = Response::new()
            //     .add_attribute("action", "burn")
            //     .add_attribute("from", info.sender)
            //     .add_attribute("amount", amount);
            // Ok(res)


            Ok(Response::new())
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
    })?;

        
    // clear all allowances that will never be used again
    ALLOWANCES.clear(deps.storage);
    ALLOWANCES_SPENDER.clear(deps.storage);
    
    Ok(Response::default())
}