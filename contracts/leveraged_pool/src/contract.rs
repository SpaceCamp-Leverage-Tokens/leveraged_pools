#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    to_binary };
use cw0::{maybe_addr};
use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, QueryMsg, HyperparametersResponse,
    PoolStateResponse , AllPoolInfoResponse};
use crate::state::{HYPERPARAMETERS, Hyperparameters, PoolState, POOLSTATE};
use crate::swap::{TSLiason, TSPricePoint};

/**
 * Instantiation entrypoint
 */
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    /* Validate that terraswap pair address is at least valid */
    let terraswap_pair_addr = maybe_addr(
        deps.api,
        Some(msg.terraswap_pair_addr)
    )?.unwrap();

    /* Validate that leveraged asset address is at least valid */
    let leveraged_asset_addr = maybe_addr(
        deps.api,
        Some(msg.leveraged_asset_addr)
    )?.unwrap();

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

    /* Fetch current TS price */
    let l: TSLiason = TSLiason::new_from_pair(
        &hyper_p.terraswap_pair_addr,
        &hyper_p.leveraged_asset_addr
    );

    let opening_price = l.fetch_ts_price(&env, deps.as_ref())?;
   
    let leveraged_opening_price = TSPricePoint{
        u_price: opening_price.u_price,
        timestamp: opening_price.timestamp,
    };
    /* Initialize pool state */
    // Initializes leveraged price to equal the leveraged price

    let init_state = PoolState {
        asset_opening_price: opening_price,
        leveraged_opening_price: leveraged_opening_price,
        assets_in_reserve: 0,
        total_leveraged_assets: 0,
        total_asset_pool_share: 0,
        total_leveraged_pool_share: 0,
    };

    /* Save for our reference across contract lifetime */
    HYPERPARAMETERS.save(deps.storage, &hyper_p)?;
    POOLSTATE.save(deps.storage, &init_state)?;

    Ok(Response::new()
       .add_attribute("method", "instantiate"))
}

/**
 * Execution entrypoint
 */
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    /* TODO */
    Err(ContractError::Unimplemented { })
}

/**
 * Expose immutable hyperparameters configured at init time
 * QueryMsg::HyperParameters
 */
fn query_hyperparameters(deps: Deps) -> StdResult<HyperparametersResponse> {
    let hyper_p = HYPERPARAMETERS.load(deps.storage)?;

    /* This never fails */
    Ok(HyperparametersResponse {
        leverage_amount:hyper_p.leverage_amount,
        minimum_protocol_ratio: hyper_p.minimum_protocol_ratio,
        rebalance_ratio: hyper_p.rebalance_ratio,
        mint_premium: hyper_p.mint_premium,
        rebalance_premium: hyper_p.rebalance_premium,
        terraswap_pair_addr: hyper_p.terraswap_pair_addr.into(),
        leveraged_asset_addr: hyper_p.leveraged_asset_addr.into(),
    })
}

/**
 * QueryMsg::PoolState
 */
fn query_pool_info(deps: Deps) -> StdResult<PoolStateResponse> {
    let pool_state = POOLSTATE.load(deps.storage)?;
    Ok(PoolStateResponse{
        asset_opening_price: pool_state.asset_opening_price,
        leveraged_opening_price: pool_state.leveraged_opening_price,
        assets_in_reserve: pool_state.assets_in_reserve,
        total_leveraged_assets: pool_state.total_leveraged_assets,
        total_asset_pool_share: pool_state.total_asset_pool_share,
        total_leveraged_pool_share: pool_state.total_leveraged_pool_share,
    })
}

/**
 * QueryMsg::PoolState
 */
fn query_all_pool_info(deps: Deps) -> StdResult<AllPoolInfoResponse> {
    let pool_state = POOLSTATE.load(deps.storage)?;
    let pool_response = PoolStateResponse{
        asset_opening_price: pool_state.asset_opening_price,
        leveraged_opening_price: pool_state.leveraged_opening_price,
        assets_in_reserve: pool_state.assets_in_reserve,
        total_leveraged_assets: pool_state.total_leveraged_assets,
        total_asset_pool_share: pool_state.total_asset_pool_share,
        total_leveraged_pool_share: pool_state.total_leveraged_pool_share,
    };

    let hyper_p = HYPERPARAMETERS.load(deps.storage)?;
    let hyper_response = HyperparametersResponse {
        leverage_amount:hyper_p.leverage_amount,
        minimum_protocol_ratio: hyper_p.minimum_protocol_ratio,
        rebalance_ratio: hyper_p.rebalance_ratio,
        mint_premium: hyper_p.mint_premium,
        rebalance_premium: hyper_p.rebalance_premium,
        terraswap_pair_addr: hyper_p.terraswap_pair_addr.into(),
        leveraged_asset_addr: hyper_p.leveraged_asset_addr.into(),
    };

    Ok(AllPoolInfoResponse{
        pool_state: pool_response,
        hyperparameters: hyper_response,
    })
}

/**
 * Query entrypoint
 */
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Hyperparameters { } => to_binary(&query_hyperparameters(deps)?),
        QueryMsg::PoolState { } => to_binary(&query_pool_info(deps)?),
        QueryMsg::AllPoolInfo { } => to_binary(&query_all_pool_info(deps)?),
    }
}

