use cosmwasm_schema::cw_serde;
// use cosmwasm_schema::cw_serde;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub core_address: Option<String>,
}

pub const STATE: Item<Config> = Item::new("config");
