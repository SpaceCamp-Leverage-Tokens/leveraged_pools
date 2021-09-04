/*
 * Liquidity manager
 *
 * Provides liquidity deposits and withdrawals
 */
use cosmwasm_std::{
    StdError, Uint128, DepsMut, MessageInfo, Env, CosmosMsg,
    SubMsg, WasmMsg, Response,Reply, ReplyOn, StdResult, to_binary
};
use cosmwasm_std::entry_point;
use crate::error::ContractError;
// use cw_storage_plus::{Item, Map};
use cw20::{Cw20ExecuteMsg};
use leveraged_pools::pool::{ProvideLiquidityMsg};
// use crate::{leverage_man};
// use crate::swap::TSLiason;


pub struct ProviderPosition {
    pub asset_pool_partial_share: Uint128,
    pub asset_pool_total_share: Uint128,
}

pub fn try_execute_provide_liquidity(
    _deps: DepsMut,
    _info: MessageInfo,
    env: &Env,
    msg: ProvideLiquidityMsg,
) -> Result<Response, ContractError> {

    // let provider_position = leverage_man::get_liquidity_map(&deps,info.sender);
    // let price_context = leverage_man::get_price_context(&deps, &env, deps.querier)?;

    // let added_value = price_context.current_snapshot.asset_price.saturating_mul() ;
    // Get total asset share form leverage_man?
    // Get mapping from Addr to asset_partial_share from leverage_man

    let cw20_msg = Cw20ExecuteMsg::Transfer{
        amount: msg.amount,
        recipient: env.contract.address.to_string()
    };
    // Err(ContractError::Unimplemented {})
    //Accept funds from user
    //Update total asset share 

    //Update partial share mapping 

    Ok(Response::new().add_submessage(SubMsg {
            msg: CosmosMsg::Wasm( WasmMsg::Execute{
                contract_addr: msg.token.to_string(),
                msg:to_binary(&cw20_msg)?,
                funds:vec![]
            } )
            .into(),
            gas_limit: None,
            id: 1,
            reply_on: ReplyOn::Success,
        }))
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

    match msg.id {
        1=> {
            // Err(StdError::generic_err("Execute message failed"))
            Ok(Response::new())
        }
        _ => Err(StdError::generic_err("reply id is invalid"))
    }    
}