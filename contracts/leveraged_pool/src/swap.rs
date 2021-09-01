use std::vec::{Vec};
use crate::error::ContractError;
use cosmwasm_std::{Env, Addr, to_binary, QueryRequest, WasmQuery};
use cosmwasm_std::{Uint128, QuerierWrapper, Storage};
use terraswap::pair::{
    QueryMsg as TerraSwapPairQueryMsg,
    PoolResponse as TerraSwapPoolResponse };
use cw_storage_plus::Item;
use terraswap::asset::{AssetInfo};
use leveraged_pools::pool::{TSPricePoint};

pub const PRICE_DATA_EXPIRY: u64 = 15 * 60;
pub const PRICE_DATA_N: usize = 90 * 24 * 4;
/**
 * TerraSwap liason for querying and eventually swapping
 */
pub struct TSLiason {
    pool: Addr,
    leveraged_asset: Addr,
    asset_price_data_key: String,
    /* TODO trigger leveraged asset price calculation when we fetch TS price
     * ... that struct should have ownership over [leveraged and normal] asset
     * price histories
     * leverage_manager: LeverageManager
     */
}

impl TSLiason {
    pub fn new_from_pair(n_pool: &Addr, n_asset: &Addr) -> Self {
        /* Construct unique key with which we'll retrieve pair price history */
        let mut storage_name = String::from("asset_price_data");
        storage_name.push_str(n_pool.as_str());
        storage_name.push_str(n_asset.as_str());

        TSLiason {
            pool: Addr::unchecked(n_pool.as_str()),
            leveraged_asset: Addr::unchecked(n_asset.as_str()),
            asset_price_data_key: storage_name,
        }
    }

    /* Query given a single TS pool for current price */
    pub fn fetch_ts_price(&self, env: &Env, querier: QuerierWrapper, storage: &mut dyn Storage) -> Result<TSPricePoint, ContractError> {
        /* Query TS contract */
        let res: TerraSwapPoolResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: (*self.pool.as_str()).to_string(),
            msg: to_binary(&TerraSwapPairQueryMsg::Pool { })?,
        }))?;

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

        /* Load this pair's historic price data */
        let asset_price_data_item = Item::new(&self.asset_price_data_key);
        let mut price_data: Vec<TSPricePoint>;
        match asset_price_data_item.load(storage) {
            /* Loaded successfully */
            Ok(data) => { price_data = data },

            /* Create if not existing already */
            _ => { price_data = Vec::new(); },
        };

        /* Update historic price data if most recent is too old */
        match price_data.last() {
            Some(&price) => {
                if price_timestamp_expired(&price, env) {
                    push_next_price(&mut price_data, current_price.clone());
                }
            }

            /* 0-length vector -> no price history indexed yet */
            _ => {
                push_next_price(&mut price_data, current_price.clone());
            }
        }

        asset_price_data_item.save(storage, &price_data)?;

        /* Return price data point */
        Ok(current_price)
    }

    pub fn asset_price_history(&self, storage: &dyn Storage) -> Vec<TSPricePoint> {
        /* Load this pair's historic price data */
        let asset_price_data = Item::new(&self.asset_price_data_key);
        match asset_price_data.load(storage) {
            /* Loaded successfully */
            Ok(data) => { return data; },

            /* No history */
            _ => { return Vec::new(); },
        };
    }
}

fn price_timestamp_expired(price_point: &TSPricePoint, env: &Env) -> bool {
    let currently = env.block.time.seconds();
    let timestamp = price_point.timestamp;

    currently > timestamp && currently - timestamp > PRICE_DATA_EXPIRY
}

fn push_next_price(price_data: &mut std::vec::Vec<TSPricePoint>, price: TSPricePoint) {
    /* Append most recent price */
    price_data.push(price);

    let len = price_data.len();
    if len > PRICE_DATA_N {
        /* Pop old price data off the front */
        price_data.drain(0..len - PRICE_DATA_N - 1);
    }
}
