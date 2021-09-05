/*
 * Liquidity manager
 *
 * Provides liquidity deposits and withdrawals
 */
use cosmwasm_std::{
    StdError, Uint128, DepsMut, MessageInfo, Env,
    SubMsg, Response,Reply ,StdResult, 
};
use cosmwasm_std::entry_point;
use crate::error::ContractError;
// use cw_storage_plus::{Item, Map};
// use cw20::{Cw20ExecuteMsg};
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
    _env: &Env,
    _msg: ProvideLiquidityMsg,
) -> Result<Response, StdError> {

    // let provider_position = leverage_man::get_liquidity_map(&deps,info.sender);
    // let price_context = leverage_man::get_price_context(&deps, &env, deps.querier)?;

    // let added_value = price_context.current_snapshot.asset_price.saturating_mul() ;
    // Get total asset share form leverage_man?
    // Get mapping from Addr to asset_partial_share from leverage_man

    // let cw20_msg = Cw20ExecuteMsg::TransferFrom{
    //     owner: info.sender.to_string(),
    //     amount: msg.amount,
    //     recipient: env.contract.address.to_string()
    // };

    let mut _messages: Vec<SubMsg> = vec![];

    // Transfer Funds 
    // messages.push(
    //     SubMsg {
    //     msg: CosmosMsg::Wasm(WasmMsg::Execute {
    //     contract_addr: msg.token.to_string(),
    //     msg: to_binary(&Cw20ExecuteMsg::Transfer {
    //         // owner: info.sender.to_string(),
    //         recipient: env.contract.address.to_string(),
    //         amount: msg.amount,
    //     })?,
    //     funds: vec![],
    //     }),
    //     gas_limit:None,
    //     id:1,
    //     reply_on: ReplyOn::Always });

    if true {
        // return Err(StdError::generic_err(msg.amount.to_string())) 
    }
    
    // Err(ContractError::Unimplemented {})
    //Accept funds from user
    //Update total asset share 

    //ADD MESSAGE UPDATING USER SHARE 

    Ok(Response::new())
    // Ok(Response::new().add_submessages(msgs: impl IntoIterator<Item = SubMsg<T>>))

}

// fn update_liquidity_state_add()-> <Result<Response, ContractError> {
//     Err(ContractError::Unimplemented{ })
// }

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
