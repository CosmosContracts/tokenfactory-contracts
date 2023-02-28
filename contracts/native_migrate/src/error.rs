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

    #[error("You have to send at least 1 denom")]
    NoFundsSent {},

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
