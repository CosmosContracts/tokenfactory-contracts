use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid denom: {denom:?} {message:?}")]
    InvalidDenom { denom: String, message: String },

    #[error("this is not an invalid cw20 message")]
    InvalidCW20Message {},

    #[error("invalid cw20 address, does not match with state.")]
    InvalidCW20Address {},

    #[error("This contract does not have enough funds to cover {request:?}. It only has {amount:?} currently.")]
    OutOfFunds { request: Uint128, amount: Uint128 },

    #[error("{message:?}. {mode:?}.")]
    InvalidMode { mode: String, message: String },

    #[error(
        "This contract does not have enough balance. Please contact an admin / support: {denom:?}."
    )]
    InsufficientContractBalance {
        denom: String,
        balance: Uint128,
        required: Uint128,
    },

    #[error("{message:?}")]
    InvalidMinterAddress { message: String },
}
