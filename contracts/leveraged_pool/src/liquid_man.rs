/*
 * Liquidity manager
 *
 * Provides liquidity deposits and withdrawals
 */
use cosmwasm_std::{
    StdError, Uint128, DepsMut, MessageInfo, Env,Response,Reply ,StdResult, 
};
use cosmwasm_std::entry_point;
use crate::error::ContractError;
use leveraged_pools::pool::{ProvideLiquidityMsg};
use crate::{leverage_man};

pub struct ProviderPosition {
    pub asset_pool_partial_share: Uint128,
    pub asset_pool_total_share: Uint128,
}

pub fn try_execute_provide_liquidity(
    deps: DepsMut,
    _info: MessageInfo,
    env: &Env,
    msg: ProvideLiquidityMsg,
) -> Result<Response, ContractError> {

    let mut pool_state = leverage_man::get_pool_state(&deps.as_ref())?;
    let provider_position = leverage_man::get_liquidity_map(&deps.as_ref(),&msg.sender)?;
    let price_context = leverage_man::get_price_context(&deps.as_ref(), env, deps.querier)?;
    
    let liquidity_value_added = msg.amount.saturating_mul(price_context.current_snapshot.asset_price); 
    let new_provider_position = provider_position.asset_pool_partial_share + liquidity_value_added;
    
    pool_state.assets_in_reserve = pool_state.assets_in_reserve + msg.amount;
    pool_state.total_asset_pool_share = pool_state.total_asset_pool_share + liquidity_value_added;

    leverage_man::update_pool_state(deps.storage, pool_state)?;
    leverage_man::update_pool_share(deps.storage, &msg.sender, &new_provider_position)
    
}

pub fn execute_withdraw_liquidity(
    _deps: DepsMut,
    _info: MessageInfo,
) -> Result<ProviderPosition, ContractError> {
    Err(ContractError::Unimplemented{ })
}


/// This just stores the result for future query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {

    if true{
        return Err(StdError::generic_err("Reply was successful but wtf")) 
    }

    match msg.id {
        1=> {
            // Err(StdError::generic_err("Execute message failed"))
            Ok(Response::new())
        }
        _ => Err(StdError::generic_err("reply id is invalid"))
    }    
}