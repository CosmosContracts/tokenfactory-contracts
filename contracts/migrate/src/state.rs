use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct State {
    pub contract_minter_address: String,

    /// if not set, must set burn_denom
    pub cw20_token_address: Option<String>,
    /// if not set, must set cw20_token_address
    pub burn_denom: Option<String>,

    pub tf_denom: String,
}

pub const STATE: Item<State> = Item::new("state");
