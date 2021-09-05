/*
 * Liquidity manager
 *
 * Provides liquidity deposits and withdrawals
 */
use cosmwasm_std::{
    StdError, Uint128, DepsMut, MessageInfo, Env,Response,Reply ,StdResult, WasmMsg, to_binary,CosmosMsg
};
use cosmwasm_std::entry_point;
use crate::error::ContractError;
use leveraged_pools::pool::{ProvideLiquidityMsg};
use crate::{leverage_man};

use cw20::Cw20ExecuteMsg;

pub struct ProviderPosition {
    pub asset_pool_partial_share: Uint128,
    pub asset_pool_total_share: Uint128,
}

pub fn try_execute_provide_liquidity(
    deps: DepsMut,
    _info: MessageInfo,
    _env: &Env,
    msg: ProvideLiquidityMsg,
) -> Result<Response, ContractError> {

    let mut pool_state = leverage_man::get_pool_state(&deps.as_ref())?;
    let provider_position = leverage_man::get_liquidity_position(&deps.as_ref(),&msg.sender)?;
    // let price_context = leverage_man::get_price_context(&deps.as_ref(), env, deps.querier)?;
    
    let liquidity_value_added = msg.amount; 
    let new_provider_position = provider_position.asset_pool_partial_share + liquidity_value_added;
    
    pool_state.assets_in_reserve = pool_state.assets_in_reserve + msg.amount;
    pool_state.total_asset_pool_share = pool_state.total_asset_pool_share + liquidity_value_added;

    
    leverage_man::update_pool_share(deps.storage, &msg.sender, &new_provider_position)?;
    leverage_man::update_pool_state(deps.storage, pool_state)?;
    
    Ok(Response::new())
}

pub fn execute_withdraw_liquidity(
    deps: DepsMut,
    info: MessageInfo,
    env: &Env,
    requested_share_of_pool: Uint128,
) -> Result<Response, ContractError> {
    let provider_position = leverage_man::get_liquidity_position(&deps.as_ref(), &info.sender)?;

    // If requesting more than put into the pool
    
    let hyper_p = leverage_man::query_hyperparameters(&deps.as_ref())?;

    if requested_share_of_pool > provider_position.asset_pool_partial_share{
        return Err(ContractError::InsuficientFunds{})
    }

    let mut pool_state = leverage_man::get_pool_state(&deps.as_ref())?;
    let price_context = leverage_man::get_price_context(&deps.as_ref(), env, deps.querier)?;

    let total_asset_value  = pool_state.assets_in_reserve.saturating_mul(price_context.current_snapshot.asset_price);
    let total_minted_value = pool_state.total_leveraged_assets.saturating_mul(price_context.current_snapshot.leveraged_price);

    let total_liq_pool_value = total_asset_value - total_minted_value;
    let available_pool_tokens  = total_liq_pool_value/price_context.current_snapshot.asset_price;

    let percent_pool_requested = Uint128::new(1_000_000).saturating_mul(requested_share_of_pool)/pool_state.total_asset_pool_share;
    let tokens_requested = available_pool_tokens.saturating_mul(percent_pool_requested)/Uint128::new(1_000_000);
    
    let new_assets_in_reserve = pool_state.assets_in_reserve - tokens_requested;

    // If nothing minted -- user can withdraw any amount
    if total_minted_value > Uint128::new(0){
        // Calculating next pool state validity
        let new_total_asset_value = new_assets_in_reserve.saturating_mul(price_context.current_snapshot.asset_price);
        let new_total_liq_pool_value = new_total_asset_value - total_minted_value;
        let new_protocol_ratio = Uint128::new(1_000_000).saturating_mul(new_total_liq_pool_value)/total_minted_value;

        // Ensure that new protocol ratio is above the rebalance ratio
        if new_protocol_ratio < hyper_p.rebalance_ratio {
            return Err(ContractError::InvalidPoolState{ })
        }
    }  

    let new_provider_position = provider_position.asset_pool_partial_share - requested_share_of_pool;
    pool_state.assets_in_reserve = new_assets_in_reserve; 
    pool_state.total_asset_pool_share = pool_state.total_asset_pool_share - requested_share_of_pool; 
    
    // Update Pool State
    leverage_man::update_pool_state(deps.storage, pool_state)?;
    leverage_man::update_pool_share(deps.storage, &info.sender, &new_provider_position)?;

    // if requested_share_of_pool < provider_position.asset_pool_partial_share{
    //     return Err(ContractError::Std(StdError::generic_err(deps.api.addr_humanize(&hyper_p.leveraged_asset_addr)?.to_string())))
    // }
    
    let request_tokens_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&hyper_p.leveraged_asset_addr)?.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: info.sender.to_string(),
                amount: tokens_requested,
            })?,
    });

    Ok(Response::new().add_message(request_tokens_msg))
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
