use std::str::FromStr;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cw20::Cw20ReceiveMsg;

use strum_macros::Display;

#[cw_serde]
#[derive(Display)]
pub enum Mode {
    #[strum(serialize = "mint")]
    Mint,
    #[strum(serialize = "balance")]
    Balance,
}

impl Eq for Mode {}

impl FromStr for Mode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mint" => Ok(Mode::Mint),
            "balance" => Ok(Mode::Balance),
            _ => Err(format!("Invalid mode: {}", s)),
        }
    }
}

#[cw_serde]
pub struct InstantiateMsg {
    pub mode: String, // "balance" or "mint". If "mint", contract_minter_address is required

    pub tf_denom: String,
    pub cw20_token_address: String,

    pub contract_minter_address: Option<String>, // core middleware contract. not required if you have mod=balance
}

#[cw_serde]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetConfig)]
    GetConfig {},
}

// We define a custom struct for each query response
#[cw_serde]
pub struct GetConfig {
    pub contract_minter_address: Option<String>,
    pub cw20_token_address: String,
    pub tf_denom: String,
    pub mode: String,
}
