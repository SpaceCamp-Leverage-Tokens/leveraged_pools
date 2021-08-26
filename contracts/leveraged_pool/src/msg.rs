use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    HyperParameters {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HyperParametersResponse {
    pub leverage_amount: u32,
    pub minimum_protocol_ratio: u32,
    pub rebalance_ratio: u32,
    pub mint_premium: u32,
    pub rebalance_premium: u32,
    pub terraswap_pair_addr: String,
    pub leveraged_asset_addr: String,
}

