/*
 * Provides mocked responses from TerraSwap pools for testing
 *
 * Contents mostly from a file I originally found in Mirror protocol repo. All
 * due credit to them!
 *
 * https://github.com/Mirror-Protocol/mirror-contracts/blob/main/contracts/mirror_collateral_oracle/src/testing/mock_querier.rs
 * Apache License
 * Version 2.0, January 2004
 */

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::{
    MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR,
};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Coin, ContractResult, OwnedDeps,
    Querier, QuerierResult, QueryRequest, SystemError, SystemResult, Uint128,
    WasmQuery,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use terra_cosmwasm::TerraQueryWrapper;
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::PoolResponse;

pub type OwnedMockDeps = OwnedDeps<MockStorage, MockApi, WasmMockQuerier>;

/// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
/// which emulates terraswap pools
pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier: WasmMockQuerier = WasmMockQuerier::new(
        MockQuerier::new(&[(MOCK_CONTRACT_ADDR, contract_balance)]),
    );

    OwnedDeps {
        api: MockApi::default(),
        storage: MockStorage::default(),
        querier: custom_querier,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier<TerraQueryWrapper>,
    terraswap_pools_querier: TerraswapPoolsQuerier,
}

#[derive(Clone, Default)]
pub struct TerraswapPoolsQuerier {
    pools: HashMap<String, (String, Uint128, String, Uint128)>,
}

impl TerraswapPoolsQuerier {
    #[allow(clippy::type_complexity)]
    pub fn new(
        pools: &[(&String, (&String, &Uint128, &String, &Uint128))],
    ) -> Self {
        TerraswapPoolsQuerier {
            pools: pools_to_map(pools),
        }
    }
}

#[allow(clippy::type_complexity)]
pub(crate) fn pools_to_map(
    pools: &[(&String, (&String, &Uint128, &String, &Uint128))],
) -> HashMap<String, (String, Uint128, String, Uint128)> {
    let mut pools_map: HashMap<String, (String, Uint128, String, Uint128)> =
        HashMap::new();
    for (key, pool) in pools.iter() {
        pools_map.insert(
            key.to_string(),
            (pool.0.clone(), *pool.1, pool.2.clone(), *pool.3),
        );
    }
    pools_map
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely here
        let request: QueryRequest<TerraQueryWrapper> =
            match from_slice(bin_request) {
                Ok(v) => v,
                Err(e) => {
                    return SystemResult::Err(SystemError::InvalidRequest {
                        error: format!("Parsing query request: {}", e),
                        request: bin_request.into(),
                    })
                }
            };
        self.handle_query(&request)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ReferenceData {
    rate: Uint128,
    last_updated_base: u64,
    last_updated_quote: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct EpochStateResponse {
    exchange_rate: Decimal256,
    aterra_supply: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Pool {},
    GetReferenceData {
        base_symbol: String,
        quote_symbol: String,
    },
    EpochState {
        block_height: Option<u64>,
        distributed_interest: Option<Uint256>,
    },
}

impl WasmMockQuerier {
    pub fn handle_query(
        &self,
        request: &QueryRequest<TerraQueryWrapper>,
    ) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => {
                match from_binary(msg).unwrap() {
                    QueryMsg::Pool {} => match self
                        .terraswap_pools_querier
                        .pools
                        .get(contract_addr)
                    {
                        Some(v) => SystemResult::Ok(ContractResult::from(
                            to_binary(&PoolResponse {
                                assets: [
                                    Asset {
                                        amount: v.1,
                                        info: AssetInfo::NativeToken {
                                            denom: v.0.clone(),
                                        },
                                    },
                                    Asset {
                                        amount: v.3,
                                        info: AssetInfo::Token {
                                            contract_addr: v.2.to_string(),
                                        },
                                    },
                                ],
                                total_share: Uint128::zero(),
                            }),
                        )),
                        None => {
                            SystemResult::Err(SystemError::InvalidRequest {
                                error: "No pair info exists".to_string(),
                                request: msg.as_slice().into(),
                            })
                        }
                    },
                    QueryMsg::GetReferenceData { .. } => SystemResult::Ok(
                        ContractResult::from(to_binary(&ReferenceData {
                            rate: Uint128::from(3465211050000000000000u128),
                            last_updated_base: 100u64,
                            last_updated_quote: 100u64,
                        })),
                    ),
                    QueryMsg::EpochState { .. } => SystemResult::Ok(
                        ContractResult::from(to_binary(&EpochStateResponse {
                            exchange_rate: Decimal256::from_ratio(10, 3),
                            aterra_supply: Uint256::from_str("123123123")
                                .unwrap(),
                        })),
                    ),
                }
            }
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<TerraQueryWrapper>) -> Self {
        WasmMockQuerier {
            base,
            terraswap_pools_querier: TerraswapPoolsQuerier::default(),
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn with_terraswap_pools(
        &mut self,
        pairs: &[(&String, (&String, &Uint128, &String, &Uint128))],
    ) {
        self.terraswap_pools_querier = TerraswapPoolsQuerier::new(pairs);
    }
}
