use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

use cw20::{AllowanceResponse, Logo, MarketingInfoResponse};

#[cw_serde]
pub struct TokenInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: Uint128,
    pub mint: Option<MinterData>,
}

#[cw_serde]
pub struct MinterData {
    pub minter: Addr,
    /// cap is how many more tokens can be issued by the minter
    pub cap: Option<Uint128>,
}

impl TokenInfo {
    pub fn get_cap(&self) -> Option<Uint128> {
        self.mint.as_ref().and_then(|v| v.cap)
    }
}

#[cw_serde]
pub struct MigrateConfig {
    pub tf_core_address: String,
    pub tf_denom: String,
    pub start_after_addr: Option<String>,

    pub burn_cw20_balances: bool,
}

/// Stores the contract configuration at the given key
pub const MIGRATION: Item<MigrateConfig> = Item::new("migration");

pub const TOKEN_INFO: Item<TokenInfo> = Item::new("token_info");
pub const MARKETING_INFO: Item<MarketingInfoResponse> = Item::new("marketing_info");
pub const LOGO: Item<Logo> = Item::new("logo");

pub const BALANCES: Map<&Addr, Uint128> = Map::new("balance");

// we clear these on migrate
pub const ALLOWANCES: Map<(&Addr, &Addr), AllowanceResponse> = Map::new("allowance");
pub const ALLOWANCES_SPENDER: Map<(&Addr, &Addr), AllowanceResponse> =
    Map::new("allowance_spender");
