use cosmwasm_schema::{cw_serde, QueryResponses};

use cosmwasm_std::Coin;

#[cw_serde]
pub struct InstantiateMsg {
    // Assuming we handle all the denoms in 1 contract, we put that here.
    pub core_factory_address: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    MintTokens {
        core_factory_address: Option<String>, // handled in state.rs now

        denoms: Vec<Coin>,
        // denoms: String,
        to_address: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::Config)]
    GetConfig {},
}
