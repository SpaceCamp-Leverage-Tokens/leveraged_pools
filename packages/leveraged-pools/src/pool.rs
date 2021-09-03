use std::vec::{Vec};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Uint128, Addr};

/**
 * Timestamp in seconds since 1970-01-01T00:00:00Z
 */
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, JsonSchema)]
pub struct TSPricePoint {
    pub u_price: Uint128,
    pub timestamp: u64,
}

/* Snapshot of leveraged vs unleveraged price at a given time */
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, JsonSchema)]
pub struct PriceSnapshot {
    /* Price of the unleveraged asset */
    pub asset_price: Uint128,

    /* Derived price of leveraged asset */
    pub leveraged_price: Uint128,

    /* Time of this snapshot in seconds since 1970-01-01T00:00:00Z */
    pub timestamp: u64,
}

/**
 * Hyperparameter init
 */
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub leverage_amount: Uint128,
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
    ProvideLiquidity { provide_liquidity_msg:ProvideLiquidityMsg },
    WithdrawLiquidity { },
    MintLeveragedAsset { },
    BurnLeveragedAsset { },
    SetDailyLeverageReference { },
}

/**
 * Response to withdrawal / deposit of liquidity
 */
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct LiquidityResponse {
    // TODO IDK probably pool_share or something
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MinterPosition {
    pub asset_pool_partial_share: Uint128,
    pub leveraged_pool_partial_share: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ProvideLiquidityMsg {
    pub token:Addr,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ProviderPosition {
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Hyperparameters { },
    PoolState { },
    AllPoolInfo { },
    PriceHistory { },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PriceHistoryResponse {
    pub price_history: Vec<PriceSnapshot>,
}

/**
 * If no parameters were adjusted over the contract lifetime these are the
 * values the contract was initialized with
 */
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HyperparametersResponse {
    pub leverage_amount: Uint128,
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
    /**
     * Price at "opening" (since leverage was reset)
     */
    pub opening_snapshot: PriceSnapshot,

    /**
     * Backing assets provided by both minters and providers
     */
    pub assets_in_reserve: u32,

    /**
     * Minted assets
     */
    pub total_leveraged_assets: u32,

    /**
     * Total share of all assets
     *
     * TODO is this just assets_in_reserve?
     */
    pub total_asset_pool_share: Uint128,

    /**
     * Total share of all minted leveraged assets
     *
     * TODO is this just total_leveraged_assets?
     */
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

