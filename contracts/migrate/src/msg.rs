use cosmwasm_schema::{cw_serde, QueryResponses};
use cw20::Cw20ReceiveMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub contract_minter_address: String, // core middleware contract

    /// if not set, must set burn_denom
    pub cw20_token_address: Option<String>,
    /// if not set, must set cw20_token_address
    pub burn_denom: Option<String>,

    pub tf_denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    // Receives a CW20 send via the CW20's contract and mints the token
    Receive(Cw20ReceiveMsg),
    /// Converts a standard denom amount to the new token factory denom in a 1:1 ratio
    Convert {},
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
    pub contract_minter_address: String,

    pub cw20_token_address: Option<String>,
    pub burn_denom: Option<String>,

    pub tf_denom: String,
}
