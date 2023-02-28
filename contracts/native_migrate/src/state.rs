use cosmwasm_schema::cw_serde;

use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[cw_serde]
pub struct State {
    pub contract_minter_address: Addr,
    pub burn_denom: String,
    pub tf_denom: String,
}

pub const STATE: Item<State> = Item::new("state");
