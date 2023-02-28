use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub tf_denom: String,
    pub burn_denom: String,

    pub contract_minter_address: String, // core middleware contract. not required if you have mod=balance
}

#[cw_serde]
pub enum ExecuteMsg {
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
    pub burn_denom: String,
    pub tf_denom: String,
}
