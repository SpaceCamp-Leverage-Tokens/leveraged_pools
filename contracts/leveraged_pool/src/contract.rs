use crate::error::ContractError;
use crate::{leverage_man, liquid_man, mint_man};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo,
    QuerierWrapper, Response, StdError, StdResult, Uint128,
};
use cw20::Cw20ReceiveMsg;
use leveraged_pools::pool::{
    AllPoolInfoResponse, Cw20HookMsg, ExecuteMsg, HyperparametersResponse,
    InstantiateMsg, LeveragedPositionResponse, LiquidityPositionResponse,
    PoolStateResponse, PriceHistoryResponse, ProtocolRatioResponse,
    ProvideLiquidityMsg, QueryMsg, TryBurn, TryMint,
};

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
    /* TODO do mint_man and liquid_man really need to be init'd? */
    for init in [leverage_man::init] {
        init(&env, deps.storage, deps.api, deps.querier, &msg)?;
    }

    Ok(Response::new().add_attribute("method", "instantiate"))
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
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::WithdrawLiquidity { share_of_pool } => {
            execute_withdraw_liquidity(deps, info, env, share_of_pool)
        }
        ExecuteMsg::BurnLeveragedAsset { share_of_pool } => {
            execute_burn_leveraged(deps, info, env, share_of_pool)
        }

        _ => Err(ContractError::InvalidPoolParams {}),
    }
}

/**
 * Unpack Cw20 messages
 */
pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    /* Ignore any Cw20s except the backing asset */
    if !is_pooled_asset(&deps.as_ref(), &info.sender)? {
        return Err(ContractError::WrongAssetLOL {});
    }

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::ProvideLiquidity {}) => {
            // only asset contract can execute this message
            let cw20_sender_addr = deps.api.addr_validate(&cw20_msg.sender)?;

            let provide_liquidity_msg = ProvideLiquidityMsg {
                sender: cw20_sender_addr,
                amount: cw20_msg.amount,
            };
            let _ = liquid_man::try_execute_provide_liquidity(
                deps,
                info,
                &env,
                provide_liquidity_msg,
            );

            return Ok(Response::new());
        }
        Ok(Cw20HookMsg::MintLeveragedPosition {}) => {
            execute_mint_leveraged(deps, info, &env, &cw20_msg)?;
            return Ok(Response::new());
        }
        Err(err) => Err(ContractError::Std(err)),
    }
}

/**
 * ExecuteMsg::BurnLeveragedAsset
 */
pub fn execute_burn_leveraged(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    pool_share: Uint128,
) -> Result<Response, ContractError> {
    mint_man::execute_burn_leveraged(
        deps,
        &info,
        &env,
        &TryBurn {
            sender: info.sender.clone(),
            pool_share,
        },
    )
}

fn execute_mint_leveraged(
    deps: DepsMut,
    info: MessageInfo,
    env: &Env,
    msg: &Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let sender = deps.api.addr_validate(&msg.sender)?;
    let amount = msg.amount;

    mint_man::execute_mint_leveraged(
        deps,
        &info,
        &env,
        &TryMint { sender, amount },
    )?;

    Ok(Response::new())
}

fn is_pooled_asset(deps: &Deps, addr: &Addr) -> StdResult<bool> {
    Ok(leverage_man::get_asset_addr(&deps)? == *addr)
}

/**
 * ExecuteMsg::WithdrawLiquidity
 */
pub fn execute_withdraw_liquidity(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    share_of_pool: Uint128,
) -> Result<Response, ContractError> {
    liquid_man::execute_withdraw_liquidity(deps, info, &env, share_of_pool)
}

/**
 * ExecuteMsg::ProvideLiquidity
 */
pub fn execute_provide_liquidity(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    msg: ProvideLiquidityMsg,
) -> Result<Response, ContractError> {
    liquid_man::try_execute_provide_liquidity(deps, info, &env, msg)
}

/**
 * Expose immutable hyperparameters configured at init time
 * QueryMsg::HyperParameters
 */
fn query_hyperparameters(deps: Deps) -> StdResult<HyperparametersResponse> {
    let hyper_p = leverage_man::query_hyperparameters(&deps)?;

    /* This never fails */
    Ok(HyperparametersResponse {
        leverage_amount: hyper_p.leverage_amount,
        minimum_protocol_ratio: hyper_p.minimum_protocol_ratio,
        rebalance_ratio: hyper_p.rebalance_ratio,
        mint_premium: hyper_p.mint_premium,
        rebalance_premium: hyper_p.rebalance_premium,
        terraswap_pair_addr: deps
            .api
            .addr_humanize(&hyper_p.terraswap_pair_addr)?
            .to_string(),
        leveraged_asset_addr: deps
            .api
            .addr_humanize(&hyper_p.leveraged_asset_addr)?
            .to_string(),
    })
}

/**
 * QueryMsg::PriceHistory
 */
fn query_price_history(deps: Deps) -> StdResult<PriceHistoryResponse> {
    Ok(PriceHistoryResponse {
        price_history: leverage_man::query_price_history(&deps),
    })
}

/**
 * QueryMsg::LeveragedPosition
 */
fn query_addr_leveraged_position(
    deps: Deps,
    address: Addr,
) -> StdResult<LeveragedPositionResponse> {
    Ok(LeveragedPositionResponse {
        position: leverage_man::get_leveraged_position(&deps, &address)?,
    })
}

/**
 * QueryMsg::LiquidityPosition
 */
fn query_addr_liquidity_position(
    deps: Deps,
    address: Addr,
) -> StdResult<LiquidityPositionResponse> {
    Ok(LiquidityPositionResponse {
        position: leverage_man::get_liquidity_position(&deps, &address)?,
    })
}

/**
 * QueryMsg::PoolState
 */
fn query_pool_state(deps: Deps) -> StdResult<PoolStateResponse> {
    let pool_state = leverage_man::query_pool_state(&deps)?;

    Ok(PoolStateResponse {
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
fn query_all_pool_info(
    deps: Deps,
    env: &Env,
    querier: QuerierWrapper,
) -> StdResult<AllPoolInfoResponse> {
    Ok(AllPoolInfoResponse {
        hyperparameters: query_hyperparameters(deps)?,
        pool_state: query_pool_state(deps)?,
        price_context: leverage_man::get_price_context(&deps, &env, querier)?,
    })
}

/**
 * QueryMsg::ProtocolRatio
 */
fn query_pr(
    deps: &Deps,
    env: &Env,
) -> Result<ProtocolRatioResponse, ContractError> {
    Ok(ProtocolRatioResponse {
        pr: leverage_man::query_pr(&deps, &env)?,
    })
}

/**
 * Query entrypoint
 */
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Hyperparameters {} => {
            to_binary(&query_hyperparameters(deps)?)
        }
        QueryMsg::PoolState {} => to_binary(&query_pool_state(deps)?),
        QueryMsg::AllPoolInfo {} => {
            to_binary(&query_all_pool_info(deps, &env, deps.querier)?)
        }
        QueryMsg::PriceHistory {} => to_binary(&query_price_history(deps)?),
        QueryMsg::LiquidityPosition { address } => {
            to_binary(&query_addr_liquidity_position(deps, address)?)
        }
        QueryMsg::LeveragedPosition { address } => {
            to_binary(&query_addr_leveraged_position(deps, address)?)
        }
        QueryMsg::ProtocolRatio {} => {
            to_binary(&query_pr(&deps, &env).or_else(|_|
                    /* TODO handle contracterror -> StdResult gracefully */
                    Err(StdError::GenericErr {
                        msg: String::from("No minted value"),
                    }))?)
        }
    }
}
