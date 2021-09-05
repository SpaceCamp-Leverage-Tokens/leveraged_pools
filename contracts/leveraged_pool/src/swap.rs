use crate::error::ContractError;
use cosmwasm_std::{Env, Addr, to_binary, QueryRequest, WasmQuery};
use cosmwasm_std::{Uint128, QuerierWrapper};
use terraswap::pair::{
    QueryMsg as TerraSwapPairQueryMsg,
    PoolResponse as TerraSwapPoolResponse };
use terraswap::asset::{AssetInfo};
use leveraged_pools::pool::{TSPricePoint};

/**
 * TerraSwap liason for querying and eventually swapping
 */
pub struct TSLiason {
    pool: Addr,
    leveraged_asset: Addr,
}

impl TSLiason {
    pub fn new_from_pair(n_pool: &Addr, n_asset: &Addr) -> Self {
        TSLiason {
            pool: n_pool.clone(),
            leveraged_asset: n_asset.clone(),
        }
    }

    /* Query given a single TS pool for current price */
    pub fn fetch_ts_price(&self, env: &Env, querier: QuerierWrapper) -> Result<TSPricePoint, ContractError> {
        /* Query TS contract */
        let res: TerraSwapPoolResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: (*self.pool.as_str()).to_string(),
            msg: to_binary(&TerraSwapPairQueryMsg::Pool { }).or_else(
                |e| Err(ContractError::Std(e)))?,
        })).or_else(|e| Err(ContractError::Std(e)))?;

        /* Should always return 2 assets */
        if res.assets.len() != 2 {
            return Err(ContractError::UnexpectedOracleResponse { });
        }

        /* Separate the primary asset from its denomination */
        let mut asset_amt = 0u128;
        let mut capital_amt = 0u128;
        for a in res.assets {
            match a.info {
                AssetInfo::Token { contract_addr } =>
                    if contract_addr == self.leveraged_asset.as_str() {
                        asset_amt = a.amount.u128();
                    } else {
                        capital_amt = a.amount.u128();
                    }
                AssetInfo::NativeToken { denom } =>
                    if denom == self.leveraged_asset.as_str() {
                        asset_amt = a.amount.u128();
                    } else {
                        capital_amt = a.amount.u128();
                    }
            }
        }

        /* Maybe not an error here, but I don't care either way right now */
        if asset_amt == 0u128 || capital_amt == 0u128 {
            return Err(ContractError::NoTokenLiquidity { });
        }

        /* Derive price from pool volume */
        let current_price = TSPricePoint {
            u_price: Uint128::from(capital_amt * 1_000_000 / asset_amt),
            timestamp: env.block.time.seconds(),
        };

        /* Return price data point */
        Ok(current_price)
    }
}

