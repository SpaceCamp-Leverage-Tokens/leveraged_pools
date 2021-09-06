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
    Deps, DepsMut, Env, StdResult, Storage, CanonicalAddr, Api, QuerierWrapper, Uint128, Addr,
    Response
};
use crate::error::ContractError;
use crate::swap::TSLiason;
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};
use leveraged_pools::pool::{InstantiateMsg, PriceSnapshot, ProviderPosition, MinterPosition, PriceContext};

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

    if hyperparameters_is_valid(&hyper_p){
        return Err(ContractError::InvalidPoolParams {})
    }

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
        assets_in_reserve: Uint128::zero(),
        total_leveraged_assets: Uint128::zero(),
        total_asset_pool_share: Uint128::zero(),
        total_leveraged_pool_share: Uint128::zero(),
    };

    /* Saving game data to memory card (PS2) in MEMORY CARD SLOT 1. Do not
     * remove memory card (PS2) or the controller, reset or switch off the
     * console */
    HYPERPARAMETERS.save(storage, &hyper_p)?;
    POOLSTATE.save(storage, &init_state)?;
    PRICE_DATA.save(storage, &vec!(genesis_snapshot))?;
    // LIQUIDITYSTATE.save(storage, )

    Ok(())
}

pub fn create_leveraged_position(
    storage: &mut dyn Storage,
    sender: &Addr,
    mint_count: Uint128,
    unleveraged_assets: Uint128,
) -> Result<MinterPosition, ContractError> {
    let mut state = POOLSTATE.load(storage)?;
    let already_minted = match MINTSTATE.load(storage, &sender) {
        Ok(mint) => { mint },
        _ => { Uint128::zero() },
    };

    let new_mint_count = already_minted + mint_count;

    MINTSTATE.save(storage, &sender, &new_mint_count)?;

    state.assets_in_reserve += unleveraged_assets;
    state.total_leveraged_pool_share += mint_count;
    state.total_leveraged_assets += mint_count;

    POOLSTATE.save(storage, &state)?;

    Ok(MinterPosition {
        leveraged_pool_partial_share: mint_count,
        leveraged_pool_total_share: state.total_leveraged_pool_share,
    })
}

pub fn burn_leveraged_position(
    storage: &mut dyn Storage,
    sender: &Addr,
    burn: Uint128,
    redeem: Uint128,
) -> Result<MinterPosition, ContractError> {
    let mut pool_state = POOLSTATE.load(storage)?;
    let mut curr_pos = match MINTSTATE.load(storage, &sender) {
        Ok(curr_mint) => { MinterPosition {
            leveraged_pool_partial_share: curr_mint,
            leveraged_pool_total_share: pool_state.total_leveraged_pool_share,
        } },
        _ => { return Err(ContractError::InsufficientFunds{ }) },
    };

    pool_state.assets_in_reserve -= redeem;
    pool_state.total_leveraged_pool_share -= burn;
    pool_state.total_leveraged_assets -= burn;
    curr_pos.leveraged_pool_partial_share -= burn;
    curr_pos.leveraged_pool_total_share -= burn;

    MINTSTATE.save(storage, sender, &curr_pos.leveraged_pool_partial_share)?;
    POOLSTATE.save(storage, &pool_state)?;

    Ok(curr_pos)
}

pub fn leveraged_equivalence(
    deps: &Deps,
    env: &Env,
    asset_count: Uint128
) -> Result<Uint128, ContractError> {
    let curr = get_price_context(deps, env, deps.querier)?.current_snapshot;
    Ok(asset_count.multiply_ratio(
        curr.asset_price,
        curr.leveraged_price,
    ))
}

pub fn unleveraged_equivalence(
    deps: &Deps,
    env: &Env,
    asset_count: Uint128
) -> Result<Uint128, ContractError> {
    let curr = get_price_context(deps, env, deps.querier)?.current_snapshot;
    Ok(asset_count.multiply_ratio(
        curr.leveraged_price,
        curr.asset_price,
    ))
}

pub fn calculate_pr(total_assets: Uint128, total_leveraged_assets: Uint128) 
-> Uint128 {
    total_assets.checked_div(total_leveraged_assets).unwrap_or_default()
}

pub fn get_asset_addr(deps: &Deps) -> StdResult<Addr> {
    Ok(deps.api.addr_humanize(&HYPERPARAMETERS.load(
        deps.storage)?.leveraged_asset_addr
    )?)
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

pub fn update_pool_state(storage: &mut dyn Storage, new_pool_state: PoolState) -> Result<Response, ContractError> {
    POOLSTATE.save(storage, &new_pool_state)?;
    Ok(Response::new())
}

pub fn update_pool_share(storage: &mut dyn Storage, user_addr:&Addr, user_shares: &Uint128) -> Result<Response, ContractError> {
    LIQUIDITYSTATE.save(storage, user_addr, user_shares)?;
    Ok(Response::new())
}

/**
 * Retrives a Mutable PoolState
 */
pub fn get_pool_state(deps: &Deps) -> StdResult<PoolState> {
    POOLSTATE.load(deps.storage)
}

/***
 * Retrieves Current Minted Position
 */
pub fn get_mint_map(deps: &DepsMut, addr:Addr) -> StdResult<MinterPosition> {
    let currently_in_pool = MINTSTATE.has(deps.storage, &addr);
    let mut my_partial_share = Uint128::new(0); //if no position currently open in the pool

    if currently_in_pool {
        my_partial_share = MINTSTATE.load(deps.storage, &addr)?;
    }

    let pool_state = POOLSTATE.load(deps.storage)?;
    let total_share = pool_state.total_leveraged_pool_share;

    let my_position = MinterPosition {
        leveraged_pool_partial_share: my_partial_share,
        leveraged_pool_total_share: total_share,
    };
    return Ok(my_position)
}

pub fn addr_has_adequate_leveraged_share(
    deps: &Deps,
    addr: &Addr,
    check: Uint128
) -> bool {
    match MINTSTATE.load(deps.storage, addr) {
        Ok(partial) => { partial > check },
        Err(_) => { false },
    }
}

pub fn get_addr_leveraged_share(deps: &Deps, addr: &Addr) -> Uint128 {
    match MINTSTATE.load(deps.storage, addr) {
        Ok(pos) => { pos },
        Err(_) => { Uint128::zero() },
    }
}

/**
 * Find the leveraged position (if any) held by addr
 */
pub fn get_leveraged_position(deps: &Deps, addr:&Addr) -> StdResult<MinterPosition> {
    let leveraged_pool_partial_share = get_addr_leveraged_share(&deps, &addr);

    let pool_state = query_pool_state(&deps)?;
    let leveraged_pool_total_share = pool_state.total_leveraged_pool_share;

    Ok(MinterPosition {
        leveraged_pool_partial_share,
        leveraged_pool_total_share,
    })
}

/***
 * Retrieves Current Liquidity Position
 */
pub fn get_liquidity_position(deps: &Deps, addr:&Addr) -> StdResult<ProviderPosition> {
    let mut my_partial_share = Uint128::new(0); //if no position currently open in the pool

    if LIQUIDITYSTATE.has(deps.storage, addr){
        my_partial_share = LIQUIDITYSTATE.load(deps.storage, addr)?;
    }

    let pool_state = POOLSTATE.load(deps.storage)?;
    let total_share = pool_state.total_asset_pool_share;

    let my_position = ProviderPosition{
        asset_pool_partial_share: my_partial_share,
        asset_pool_total_share: total_share,
    };
    return Ok(my_position)
}

/**
 * Retrieves snapshot of the opening prices + calcualtes the snapshot of the current up-to-date TS price snapshot
 * with leveraged price
 */
pub fn get_price_context(deps: &Deps, env:&Env, querier: QuerierWrapper) -> StdResult<PriceContext>{

    let hyper_p = HYPERPARAMETERS.load(deps.storage)?;
    let pool_state = POOLSTATE.load(deps.storage)?;

    let liason: TSLiason = TSLiason::new_from_pair(
        &deps.api.addr_humanize(&hyper_p.terraswap_pair_addr)?,
        &deps.api.addr_humanize(&hyper_p.leveraged_asset_addr)?,
    );
    let opening_asset_price = pool_state.latest_reset_snapshot.asset_price;
    let opening_leveraged_price = pool_state.latest_reset_snapshot.leveraged_price;

    let current_asset_price_ts_point = liason.fetch_ts_price(&env, querier)?;
    let current_leveraged_price = get_leveraged_price(opening_asset_price, current_asset_price_ts_point.u_price, 
        hyper_p.leverage_amount, opening_leveraged_price);

    let current_snapshot = PriceSnapshot {
        asset_price: current_asset_price_ts_point.u_price,
        leveraged_price: current_leveraged_price,
        timestamp: env.block.time.seconds(),
    };

    let context_snapshots = PriceContext{
        opening_snapshot: pool_state.latest_reset_snapshot,
        current_snapshot: current_snapshot,
    };

    Ok(context_snapshots)
}

/**
 * Inputs the opening price, leveraged amount, etc to calculate the current leveraged price
 */
fn get_leveraged_price(start_asset_price:Uint128, current_asset_price:Uint128,
     leverage_amount:Uint128, starting_leverage_price: Uint128) -> Uint128{
        // If no change
        if start_asset_price == current_asset_price{
            return starting_leverage_price
        }

        // If asset increases in value
        if start_asset_price < current_asset_price{
            let absolute_change = current_asset_price - start_asset_price;
            let percent_change = Uint128::new(1_000_000).saturating_mul(absolute_change)/start_asset_price;
            let leverage_percent_change = Uint128::new(1_000_000)+leverage_amount.saturating_mul(percent_change)/Uint128::new(1_000_000);
            
            let current_leveraged_price = starting_leverage_price.saturating_mul(leverage_percent_change)/Uint128::new(1_000_000);
            return current_leveraged_price
        }   
        
        // If asset decreases in value
        if start_asset_price > current_asset_price{
            let absolute_change = start_asset_price - current_asset_price;
            let percent_change = Uint128::new(1_000_000).saturating_mul(absolute_change)/start_asset_price;
            let leverage_percent_change = Uint128::new(1_000_000)-leverage_amount.saturating_mul(percent_change)/Uint128::new(1_000_000);
            
            let current_leveraged_price = starting_leverage_price.saturating_mul(leverage_percent_change)/Uint128::new(1_000_000);
            return current_leveraged_price
        }

        return Uint128::new(1_000_000)
        
}


/**
 * Checks for valid hyperparameters
 */
fn hyperparameters_is_valid(hyperparms:&Hyperparameters) -> bool {
    if hyperparms.minimum_protocol_ratio > hyperparms.rebalance_premium{
        return false
    }
    if hyperparms.mint_premium > Uint128::new(1_000_000){
        return false
    }
    if hyperparms.rebalance_premium > Uint128::new(0_100_000) {
        return false
    }
    if hyperparms.leverage_amount < Uint128::new(1_000_000){
        return false
    }
    return true
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
    pub leverage_amount: Uint128,
    pub minimum_protocol_ratio: Uint128,
    pub rebalance_ratio: Uint128,
    pub mint_premium: Uint128,
    pub rebalance_premium: Uint128,
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
 * Tracking minted leveraged assets and their unleveraged friends
 */
pub const MINTSTATE: Map<&Addr, Uint128> = Map::new("minted_partial_shares");

/**
 * Tracking minted leveraged assets and their unleveraged friends
 */
pub const LIQUIDITYSTATE: Map<&Addr, Uint128> = Map::new("liquidity_partial_shares");
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

// pub struct ProviderPosition {
//     pub asset_pool_partial_share: Uint128,
//     pub asset_pool_total_share: Uint128,
// }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PoolState {
    /**
     * Snapshot of the price after leverage was last reset
     */
    pub latest_reset_snapshot: PriceSnapshot,

    /**
     * Backing assets provided by both minters and providers
     */
    pub assets_in_reserve: Uint128,

    /**
     * Total share of all assets
     *
     * TODO is this just assets_in_reserve?
     */
    pub total_asset_pool_share: Uint128,

    /**
     * Total share of all minted leveraged assets
     * TODO remove in favor of total_leveraged_pool_share
     */
    pub total_leveraged_assets: Uint128,

    /**
     * Total share of all minted leveraged assets
     */
    pub total_leveraged_pool_share: Uint128,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn proper_percent_increase() {
        // Testing 50% increase with 2x leverage
        let starting_price = Uint128::new(1_000_000);
        let end_price = Uint128::new(1_500_000);
        let leverage_amount = Uint128::new(2_000_000);
        let leverage_start_price = Uint128::new(1_000_000);
        let leverage_end_price = get_leveraged_price(starting_price, end_price, leverage_amount, leverage_start_price);
        assert_eq!(Uint128::new(2_000_000),leverage_end_price);

        // Testing price constant 
        let starting_price = Uint128::new(1_000_000);
        let end_price = Uint128::new(1_500_000);
        let leverage_amount = Uint128::new(3_000_000);
        let leverage_start_price = Uint128::new(1_000_000);
        let leverage_end_price = get_leveraged_price(starting_price, end_price, leverage_amount, leverage_start_price);
        assert_eq!(Uint128::new(2_500_000),leverage_end_price);

        // Testing 10% decrease in price with 3x leverage 
        let starting_price = Uint128::new(1_000_000);
        let end_price = Uint128::new(0_900_000);
        let leverage_amount = Uint128::new(3_000_000);
        let leverage_start_price = Uint128::new(1_000_000);
        let leverage_end_price = get_leveraged_price(starting_price, end_price, leverage_amount, leverage_start_price);
        assert_eq!(Uint128::new(0_700_000),leverage_end_price);
    }
}
