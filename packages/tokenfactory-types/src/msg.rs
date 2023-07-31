use cosmwasm_schema::cw_serde;

use cosmwasm_std::Coin;
use token_bindings::Metadata;

#[cw_serde]
pub enum ExecuteMsg {
    // == ANYONE ==
    Burn {},

    // == WHITELIST ==
    // Mints actual tokens to an address (only whitelisted addresses can do this)
    Mint {
        address: String,
        denom: Vec<Coin>,
    },

    // == MANAGER ==
    BurnFrom {
        from: String,
        denom: Coin,
    },

    TransferAdmin {
        denom: String,
        new_address: String,
    },

    ForceTransfer {
        from: String,
        to: String,
        denom: Coin,
    },

    SetMetadata {
        denom: String,
        metadata: Metadata,
    },

    // Could be a DAO, normal contract, or CW4
    // Future: should we specify what name/denom an address can mint?
    AddWhitelist {
        addresses: Vec<String>,
    },
    RemoveWhitelist {
        addresses: Vec<String>,
    },

    AddDenom {
        denoms: Vec<String>,
    },
    RemoveDenom {
        denoms: Vec<String>,
    },
}
