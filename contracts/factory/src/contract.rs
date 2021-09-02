#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, 
    Response, StdResult, WasmMsg, SubMsg, Reply, ReplyOn, StdError, Addr};

use crate::error::ContractError;
use crate::msg::{PoolResponse, ExecuteMsg, InstantiateMsg, QueryMsg, LastResetResponse};
use crate::state::{State, STATE};
use crate::response::{MsgInstantiateContractResponse};
use leveraged_pools::pool::{InstantiateMsg as PoolInstantiatMsg, ExecuteMsg as PoolExecuteMsg};
use chrono::{ TimeZone, Utc, Datelike};
use std::convert::TryInto;

use protobuf::Message;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender.clone(),
        leveraged_pool_addrs: vec![],
        leveraged_pool_code_id: msg.leveraged_pool_code_id,
        timestamp: 0,
    };
    STATE.save(deps.storage, &state)?;

    // Create Gov Token
    // Create Gov Contract
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateNewPool { pool_instantiate_msg } => try_create_new_pool(deps, pool_instantiate_msg),
        ExecuteMsg::BroadcastLeverageUpdate { } => try_broadcast_daily_leverage_reference(env, deps),
    }
}

/**
 *  Broadcasting message to reset the daily leveraged price reference
 **/
pub fn try_broadcast_daily_leverage_reference(env:Env, deps: DepsMut) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    let mut messages: Vec<WasmMsg> = vec![];

    let current_timestamp = env.block.time.seconds();
    let stale_dt = Utc.timestamp( state.timestamp.try_into().unwrap(), 0);
    let current_dt = Utc.timestamp( current_timestamp.try_into().unwrap(),0);

    // If at least one calander day has not passed (Intent is trying to reset leverages at the start of the day )
    if stale_dt.day() >= current_dt.day(){
        return Err(ContractError::NotTimeToUpdate{})
    }
 
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.timestamp = current_timestamp;
        Ok(state)
    }).expect("Error");

    for pool_addr in state.leveraged_pool_addrs{
        messages.push(WasmMsg::Execute {
                contract_addr: pool_addr.to_string(),
                msg: to_binary(&PoolExecuteMsg::SetDailyLeverageReference{})?,
                funds: vec![],
            });
    }

    // Hand out gov tokens
    Ok(Response::new().add_messages(messages))
}

pub fn try_create_new_pool(deps: DepsMut, pool_instantiate_msg:PoolInstantiatMsg) -> Result<Response, ContractError> {

    // TODO: Create new pool and pass contract id to leveraged_pool_addrs
    let state = STATE.load(deps.storage)?;

    Ok(Response::new().add_submessage(SubMsg {
        // create asset token
        msg: WasmMsg::Instantiate {
            admin: None,
            code_id: state.leveraged_pool_code_id,
            funds: vec![],
            label: "".to_string(),
            msg: to_binary(&pool_instantiate_msg)?,
        }
        .into(),
        gas_limit: None,
        id: 1,
        reply_on: ReplyOn::Success,
    }))
}

/// This just stores the result for future query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    match msg.id {
        1=> {
            let res:MsgInstantiateContractResponse = Message::parse_from_bytes(
                msg.result.unwrap().data.unwrap().as_slice(),
            )
            .map_err(|_| {
                StdError::parse_err("MsgInstantiateContractResponse", "Failed to instantiate new pool")
            })?;
            let pool_addr = Addr::unchecked(res.get_contract_address());

            STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
                state.leveraged_pool_addrs.push(pool_addr);
                // Set timestamp when first pool contract is initialized
                if state.timestamp == 0 {
                    state.timestamp = env.block.time.seconds();
                }
                Ok(state)
            }).expect("Error");
            Ok(Response::new())
        }
        _ => Err(StdError::generic_err("reply id is invalid"))
    }    
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPools {} => to_binary(&query_pools(deps)?),
        QueryMsg::GetLastReset {} => to_binary(&query_last_reset(deps)?),
    }
}

fn query_last_reset(deps: Deps) -> StdResult<LastResetResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(LastResetResponse{ timestamp: state.timestamp})
}

fn query_pools(deps: Deps) -> StdResult<PoolResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(PoolResponse { pool_ids: state.leveraged_pool_addrs })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};
    use cosmwasm_std::Addr;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);
        let msg = InstantiateMsg { leveraged_pool_code_id: 10};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetPools {}).unwrap();
        let value: PoolResponse = from_binary(&res).unwrap();
        let empty_pool_list:Vec<Addr> = Vec::new();
        assert_eq!(empty_pool_list, value.pool_ids);

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetLastReset {}).unwrap();
        let value: LastResetResponse = from_binary(&res).unwrap();
        assert_eq!(0, value.timestamp);
    }

    #[test]
    fn test_broadcast_new_leverage_reference() {
        let mut deps = mock_dependencies(&[]);
        let msg = InstantiateMsg { leveraged_pool_code_id: 10};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let msg = ExecuteMsg::BroadcastLeverageUpdate { };
        let info = mock_info("creator", &coins(1000, "earth"));
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetLastReset {}).unwrap();
        let value: LastResetResponse = from_binary(&res).unwrap();
        assert_eq!(mock_env().block.time.seconds(), value.timestamp);
    }
}
