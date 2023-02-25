use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;

use crate::msg::Mode;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct State {
    pub contract_minter_address: Option<Addr>,
    pub cw20_token_address: String,
    pub tf_denom: String,

    pub mode: Mode,
}

pub const STATE: Item<State> = Item::new("state");
