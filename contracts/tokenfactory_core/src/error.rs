use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid denom: {denom:?} {message:?}")]
    InvalidDenom { denom: String, message: String },

    #[error("You did not specify any coins to mint for the user.")]
    NoDenomsProvided {},

    #[error("Invalid funds")]
    InvalidFunds {},
}
