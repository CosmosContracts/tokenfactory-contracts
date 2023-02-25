use cosmwasm_schema::cw_serde;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub manager: String,
    pub allowed_mint_addresses: Vec<String>,
    pub denoms: Vec<String>,
}

pub const STATE: Item<Config> = Item::new("config");
