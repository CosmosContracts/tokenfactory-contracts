use cosmwasm_schema::cw_serde;

use cosmwasm_std::Coin;

#[cw_serde]
pub enum ExecuteMsg {
    // Anyone
    Burn {},

    // == WHITELIST ==
    // Mints actual tokens to an address (only whitelisted addresses can do this)
    Mint { address: String, denom: Vec<Coin> },

    // == MANAGER ==
    TransferAdmin { denom: String, new_address: String },
    // Could be a DAO, normal contract, or CW4
    // Future: should we specify what name/denom an address can mint?
    AddWhitelist { addresses: Vec<String> },
    RemoveWhitelist { addresses: Vec<String> },

    AddDenom { denoms: Vec<String> },
    RemoveDenom { denoms: Vec<String> },
}
