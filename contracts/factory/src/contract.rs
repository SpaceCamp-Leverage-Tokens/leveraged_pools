#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use crate::error::ContractError;
use crate::msg::{PoolResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender.clone(),
        leveraged_pool_addrs: vec![],
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
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateNewPool { } => try_create_new_pool(deps, info),
    }
}
pub fn try_create_new_pool(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {

    // TODO: Create new pool and pass contract id to leveraged_pool_addrs

    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.leveraged_pool_addrs.push(info.sender);
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "try_create_new_pool"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPools {} => to_binary(&query_pools(deps)?),
    }
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

        let msg = InstantiateMsg { };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetPools {}).unwrap();
        let value: PoolResponse = from_binary(&res).unwrap();
        let empty_pool_list:Vec<Addr> = Vec::new();
        assert_eq!(empty_pool_list, value.pool_ids);
    }

    #[test]
    fn proper_new_pool() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg { };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::CreateNewPool {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let info = mock_info("anyone", &coins(2, "token"));
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetPools {}).unwrap();
        let value: PoolResponse = from_binary(&res).unwrap(); 
        assert_eq!(vec![info.sender], value.pool_ids);
    }
}