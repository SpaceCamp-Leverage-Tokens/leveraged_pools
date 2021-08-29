use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr};
use cw_storage_plus::Item;
use crate::swap::{TSPricePoint};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Hyperparameters {
    pub leverage_amount: u32,
    pub minimum_protocol_ratio: u32,
    pub rebalance_ratio: u32,
    pub mint_premium: u32,
    pub rebalance_premium: u32,
    pub terraswap_pair_addr: Addr,
    pub leveraged_asset_addr: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PoolState {
    pub asset_opening_price: TSPricePoint,
    pub leveraged_opening_price: TSPricePoint,

    pub assets_in_reserve: u32,
    pub total_leveraged_assets: u32,
    pub total_asset_pool_share: u32,
    pub total_leveraged_pool_share: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MinterPosition {
    pub asset_pool_partial_share: u32,
    pub leveraged_pool_partial_share: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ProviderPosition {
    pub asset_pool_partial_share: u32,
}

/**
 * Parameters which are (currently) never changed. Some parameters may be open
 * to adjustment within a tolerance via governance votes (TODO)
 */
pub const HYPERPARAMETERS: Item<Hyperparameters> = Item::new("hyperparameters");

/**
 * Tracking minted leveraged assets and their unleveraged friends
 */
pub const POOLSTATE: Item<PoolState> = Item::new("pool_state");
