use std::vec::{Vec};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Uint128};

/**
 * Timestamp in seconds since 1970-01-01T00:00:00Z
 */
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, JsonSchema)]
pub struct TSPricePoint {
    pub u_price: Uint128,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LeveragedPricePoint {
    pub asset_price: Uint128,
    pub leveraged_price: Uint128,
    pub timestamp: u64,
}

/**
 * Hyperparameter init
 */
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub leverage_amount: u32,
    pub minimum_protocol_ratio: u32,
    pub rebalance_ratio: u32,
    pub mint_premium: u32,
    pub rebalance_premium: u32,
    pub terraswap_pair_addr: String,
    pub leveraged_asset_addr: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    ProvideLiquidity { },
    WithdrawLiquidity { },
    MintLeveragedAsset { },
    BurnLeveragedAsset { },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Hyperparameters { },
    PoolState { },
    AllPoolInfo { },
    AssetPriceHistory { },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AssetPriceHistoryResponse {
    pub price_history: Vec<TSPricePoint>,
}

/**
 * If no parameters were adjusted over the contract lifetime these are the
 * values the contract was initialized with
 */
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HyperparametersResponse {
    pub leverage_amount: u32,
    pub minimum_protocol_ratio: u32,
    pub rebalance_ratio: u32,
    pub mint_premium: u32,
    pub rebalance_premium: u32,
    pub terraswap_pair_addr: String,
    pub leveraged_asset_addr: String,
}

/**
 * Operational data, changing as pool usage changes
 */
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PoolStateResponse {
    pub asset_opening_price: TSPricePoint,
    pub leveraged_opening_price: TSPricePoint,

    pub assets_in_reserve: u32,
    pub total_leveraged_assets: u32,
    pub total_asset_pool_share: u32,
    pub total_leveraged_pool_share: u32,
}

/**
 * One query to minimze entrances to blockchain
 */
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AllPoolInfoResponse {
    pub hyperparameters: HyperparametersResponse,
    pub pool_state: PoolStateResponse,
}

