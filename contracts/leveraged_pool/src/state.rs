use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MinterPosition {
    pub asset_pool_partial_share: u32,
    pub leveraged_pool_partial_share: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ProviderPosition {
    pub asset_pool_partial_share: u32,
}

