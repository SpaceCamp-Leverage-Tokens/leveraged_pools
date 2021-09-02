/*
 * Leverage manager
 *
 * Maintains health of the pool by controlling what mint_man can mint and what
 * liquid_man can withdraw. Essentially an arbiter of truth with the final say
 * in who can do what.
 *
 * Additionally it maintains a history of price data for both the underlying
 * asset and for its leveraged token.
 */

use std::vec::Vec;
use cosmwasm_std::{
    Deps, Env, StdResult, Storage, CanonicalAddr, Api, QuerierWrapper
};
use crate::error::ContractError;
use crate::swap::TSLiason;
use cw_storage_plus::{Item};
use serde::{Deserialize, Serialize};
use leveraged_pools::pool::{InstantiateMsg, PriceSnapshot};

pub fn init<'a>(
    env: &Env,
    storage: &mut dyn Storage,
    api: &dyn Api,
    querier: QuerierWrapper,
    msg: &InstantiateMsg,
) -> Result<(), ContractError> {
    /* Validate that terraswap pair address is at least valid */
    let terraswap_pair_addr = api.addr_canonicalize(
        &msg.terraswap_pair_addr,
    ).or_else(|_| Err(ContractError::InvalidAddr { }))?;

    /* Validate that leveraged asset address is at least valid */
    let leveraged_asset_addr = api.addr_canonicalize(
        &msg.leveraged_asset_addr,
    ).or_else(|_| Err(ContractError::InvalidAddr { }))?;

    /* Fetch current TS price */
    let liason: TSLiason = TSLiason::new_from_pair(
        &api.addr_humanize(&terraswap_pair_addr).or_else(
            |_| Err(ContractError::InvalidAddr { })
        )?,
        &api.addr_humanize(&leveraged_asset_addr).or_else(
            |_| Err(ContractError::InvalidAddr { })
        )?,
    );

    /* Set hyperparameters from inputs */
    let hyper_p = Hyperparameters {
        leverage_amount:msg.leverage_amount,
        minimum_protocol_ratio: msg.minimum_protocol_ratio,
        rebalance_ratio: msg.rebalance_ratio,
        mint_premium: msg.mint_premium,
        rebalance_premium: msg.rebalance_premium,
        terraswap_pair_addr,
        leveraged_asset_addr,
    };

    /* TODO I don't really care about TSPricePoint.timestamp, refactor maybe */
    let opening_price = liason.fetch_ts_price(&env, querier)?;
    let genesis_snapshot = PriceSnapshot {
        asset_price: opening_price.u_price,
        leveraged_price: opening_price.u_price,
        timestamp: env.block.time.seconds(),
    };

    /* Initialize pool state */
    let init_state = PoolState {
        latest_reset_snapshot: genesis_snapshot,
        assets_in_reserve: 0,
        total_leveraged_assets: 0,
        total_asset_pool_share: 0,
        total_leveraged_pool_share: 0,
    };

    /* Saving game data to memory card (PS2) in MEMORY CARD SLOT 1. Do not
     * remove memory card (PS2) or the controller, reset or switch off the
     * console */
    HYPERPARAMETERS.save(storage, &hyper_p)?;
    POOLSTATE.save(storage, &init_state)?;
    PRICE_DATA.save(storage, &vec!(genesis_snapshot))?;

    Ok(())
}

pub fn query_hyperparameters(deps: &Deps) -> StdResult<Hyperparameters> {
    Ok(HYPERPARAMETERS.load(deps.storage)?)
}

pub fn query_price_history(deps: &Deps) -> Vec<PriceSnapshot> {
    price_history(deps.storage)
}

pub fn query_pool_state(deps: &Deps) -> StdResult<PoolState> {
    POOLSTATE.load(deps.storage)
}

fn price_history(storage: &dyn Storage) -> Vec<PriceSnapshot> {
    PRICE_DATA.load(storage).unwrap_or(Vec::new())
}

/*
 * Snapshot of the price right at this exact second
 * TODO write a similar fn but have it update price history w/ DepsMut
 * TODO actually write this fn O(#￣▽￣)
 */
/* fn bleeding_edge_snapshot(deps: &Deps) -> StdResult<PriceSnapshot> {
    Err(StdError::GenericErr { msg: String::from("Unimplemented") })
} */

/**
 * Push an element onto the end of a vector and drop some of the front s/t
 * there are at most `usize` elements in the vector
 *
 * TODO Use O(#￣▽￣)
 */
#[allow(dead_code)]
fn push_drain<T>(v: &mut Vec<T>, append: T, max: usize) {
    /* Stick onto the end of Vec */
    v.push(append);

    if v.len() > max {
        /* Pop old data off the front */
        v.drain(0..v.len() - max - 1);
    }
}

/**
 * TODO Use O(#￣▽￣)
 */
#[allow(dead_code)]
fn price_timestamp_expired(snapshot: &PriceSnapshot, env: &Env) -> bool {
    let currently = env.block.time.seconds();
    let timestamp = snapshot.timestamp;

    currently > timestamp && currently - timestamp > PRICE_DATA_EXPIRY
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Hyperparameters {
    pub leverage_amount: u32,
    pub minimum_protocol_ratio: u32,
    pub rebalance_ratio: u32,
    pub mint_premium: u32,
    pub rebalance_premium: u32,
    pub terraswap_pair_addr: CanonicalAddr,
    pub leveraged_asset_addr: CanonicalAddr,
}

/**
 * Fetch a new price about every 15 minutes
 * TODO Use
 */
#[allow(dead_code)]
const PRICE_DATA_EXPIRY: u64 = 15 * 60;

/**
 * Reset leverage after 24 hours
 * TODO Use
 */
#[allow(dead_code)]
const LEVERAGE_EXPIRY: u64 = 24 * 60 * 60;

/**
 * Keep 90 days of price data at the 15-minute resolution
 * TODO Use
 */
#[allow(dead_code)]
const PRICE_DATA_N: usize = 90 * 24 * 4;

/**
 * Historic price data
 */
const PRICE_DATA: Item<Vec<PriceSnapshot>> = Item::new("price_data");

/**
 * Parameters which are (currently) never changed. Some parameters may be open
 * to adjustment within a tolerance via governance votes (TODO)
 */
const HYPERPARAMETERS: Item<Hyperparameters> = Item::new("hyperparameters");

/**
 * Tracking minted leveraged assets and their unleveraged friends
 */
const POOLSTATE: Item<PoolState> = Item::new("pool_state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PoolState {
    /**
     * Snapshot of the price after leverage was last reset
     */
    pub latest_reset_snapshot: PriceSnapshot,

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
    pub total_asset_pool_share: u32,

    /**
     * Total share of all minted leveraged assets
     *
     * TODO is this just total_leveraged_assets?
     */
    pub total_leveraged_pool_share: u32,
}

