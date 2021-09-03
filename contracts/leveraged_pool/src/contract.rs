#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    to_binary };
use crate::error::ContractError;
use leveraged_pools::pool::{
    ExecuteMsg, InstantiateMsg, QueryMsg, HyperparametersResponse,
    PoolStateResponse , AllPoolInfoResponse,
    PriceHistoryResponse, LiquidityResponse,ProvideLiquidityMsg };
use crate::{leverage_man,liquid_man};

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
    for init in [leverage_man::init] {
        init(&env, deps.storage, deps.api, deps.querier, &msg)?;
    }

    Ok(Response::new()
       .add_attribute("method", "instantiate"))
}

/**
 * Execution entrypoint
 */
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ProvideLiquidity { provide_liquidity_msg  } => Ok(Response::new()
            .set_data(
                to_binary(&execute_provide_liquidity(deps, info, env, provide_liquidity_msg)?
            ).or_else(|_| Err(ContractError::SerializeErr { }))?)
        ),

        ExecuteMsg::WithdrawLiquidity { } => Ok(Response::new()
            .set_data(
                to_binary(&execute_withdraw_liquidity(deps, info)?
            ).or_else(|_| Err(ContractError::SerializeErr { }))?)
        ),

        /*
         * TODO
         * MintLeveragedAsset { }
         * BurnLeveragedAsset { }
         * SetDailyLeverageReference { }
         */

        _ => { Err(ContractError::Unimplemented { }) },
    }
}

/**
 * ExecuteMsg::WithdrawLiquidity
 */
pub fn execute_withdraw_liquidity(
    deps: DepsMut,
    info: MessageInfo
) -> Result<LiquidityResponse, ContractError> {
    let _ = liquid_man::execute_withdraw_liquidity(deps, info)?;

    Err(ContractError::Unimplemented{ })
}

/**
 * ExecuteMsg::ProvideLiquidity
 */
pub fn execute_provide_liquidity(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    msg: ProvideLiquidityMsg
) -> Result<LiquidityResponse, ContractError> {
    let _ = liquid_man::try_execute_provide_liquidity(deps, info, &env, msg)?;

    Err(ContractError::Unimplemented{ })
}

/**
 * Expose immutable hyperparameters configured at init time
 * QueryMsg::HyperParameters
 */
fn query_hyperparameters(deps: Deps) -> StdResult<HyperparametersResponse> {
    let hyper_p = leverage_man::query_hyperparameters(&deps)?;

    /* This never fails */
    Ok(HyperparametersResponse {
        leverage_amount:hyper_p.leverage_amount,
        minimum_protocol_ratio: hyper_p.minimum_protocol_ratio,
        rebalance_ratio: hyper_p.rebalance_ratio,
        mint_premium: hyper_p.mint_premium,
        rebalance_premium: hyper_p.rebalance_premium,
        terraswap_pair_addr: deps.api.addr_humanize(
            &hyper_p.terraswap_pair_addr)?.to_string(),
        leveraged_asset_addr: deps.api.addr_humanize(
            &hyper_p.leveraged_asset_addr)?.to_string(),
    })
}

/**
 * QueryMsg::PriceHistory
 */
fn query_price_history(deps: Deps) -> StdResult<PriceHistoryResponse> {
    Ok(PriceHistoryResponse {
        price_history: leverage_man::query_price_history(&deps)
    })
}

/**
 * QueryMsg::PoolState
 */
fn query_pool_state(deps: Deps) -> StdResult<PoolStateResponse> {
    let pool_state = leverage_man::query_pool_state(&deps)?;

    Ok(PoolStateResponse{
        opening_snapshot: pool_state.latest_reset_snapshot,
        assets_in_reserve: pool_state.assets_in_reserve,
        total_leveraged_assets: pool_state.total_leveraged_assets,
        total_asset_pool_share: pool_state.total_asset_pool_share,
        total_leveraged_pool_share: pool_state.total_leveraged_pool_share,
    })
}

/**
 * QueryMsg::AllPoolInfo
 */
fn query_all_pool_info(deps: Deps) -> StdResult<AllPoolInfoResponse> {
    Ok(AllPoolInfoResponse {
        hyperparameters: query_hyperparameters(deps)?,
        pool_state: query_pool_state(deps)?,
    })
}

/**
 * Query entrypoint
 */
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Hyperparameters { } => to_binary(&query_hyperparameters(deps)?),
        QueryMsg::PoolState { } => to_binary(&query_pool_state(deps)?),
        QueryMsg::AllPoolInfo { } => to_binary(&query_all_pool_info(deps)?),
        QueryMsg::PriceHistory { } => to_binary(&query_price_history(deps)?),
    }
}

