use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: Addr,
    pub leveraged_pool_addrs: Vec<Addr>,
    pub leveraged_pool_code_id: u64,
}

pub const STATE: Item<State> = Item::new("state");
