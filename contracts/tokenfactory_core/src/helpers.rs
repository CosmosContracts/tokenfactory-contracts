use cosmwasm_std::{Addr, Coin};
use token_bindings::TokenMsg;

use crate::{state::Config, ContractError};

pub use tokenfactory_types::msg::ExecuteMsg::Mint;

pub fn is_whitelisted(state: Config, sender: Addr) -> Result<(), ContractError> {
    if !state.allowed_mint_addresses.contains(&sender.to_string()) {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

pub fn is_contract_manager(state: Config, sender: Addr) -> Result<(), ContractError> {
    if !state.manager.eq(&sender.to_string()) {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

/// Creates the token messages to mint factory tokens to an address (from this middleware contract)
/// If there are no denoms provided to mint (standard coins), it will return an error
///
/// You should not use this function unless you are within this contract. It is not for other contract use
/// unless you also use TokenMsg's, which is the entire point of this contract to not have to do.
pub fn mint_factory_token_messages(
    address: &String,
    denoms: &Vec<Coin>,
) -> Result<Vec<TokenMsg>, ContractError> {
    if denoms.is_empty() {
        return Err(ContractError::NoDenomsProvided {});
    }

    let msgs: Vec<TokenMsg> = denoms
        .iter()
        .filter(|d| denoms.iter().any(|d2| d2.denom == d.denom))
        .map(|d| TokenMsg::MintTokens {
            denom: d.denom.clone(),
            amount: d.amount,
            mint_to_address: address.to_string(),
        })
        .collect();

    Ok(msgs)
}

// Makes the output of a vector of denominations much pretty. In the format:
// 1000000:factory/juno1xxx/test, 1000000:factory/juno1xxx/test2
pub fn pretty_denoms_output(denoms: &Vec<Coin>) -> String {
    denoms
        .iter()
        .map(|d| format!("{}:{}", d.amount, d.denom))
        .collect::<Vec<String>>()
        .join(", ")
}
