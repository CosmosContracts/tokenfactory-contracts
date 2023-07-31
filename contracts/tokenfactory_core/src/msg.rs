use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    // the manager of the contract is the one who can transfer the admin to another address
    // Typically this should be a multisig or a DAO (https://daodao.zone/)
    // Default is the contract initializer
    pub manager: Option<String>,
    pub allowed_mint_addresses: Vec<String>,

    // We can manage multiple denoms
    pub existing_denoms: Option<Vec<String>>, // ex: factory/juno1xxx/test
    pub new_denoms: Option<Vec<NewDenom>>,
}

#[cw_serde]
pub struct NewDenom {
    pub name: String,
    pub description: Option<String>,
    pub symbol: String,
    pub decimals: u32,
    pub initial_balances: Option<Vec<InitialBalance>>,
}

#[cw_serde]
pub struct InitialBalance {
    pub address: String,
    pub amount: Uint128,
}

use cosmwasm_std::{Coin, Uint128};
pub use juno_tokenfactory_types::msg::ExecuteMsg;

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::Config)]
    GetConfig {},

    #[returns(Coin)]
    GetBalance { address: String, denom: String },

    #[returns(Vec<Coin>)]
    GetAllBalances { address: String },
}
