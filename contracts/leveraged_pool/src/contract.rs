#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    to_binary };
use cw0::{maybe_addr};
use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, QueryMsg, HyperparametersResponse };
use crate::state::{HYPERPARAMETERS, Hyperparameters};

/**
 * Instantiation entrypoint
 */
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    /* Validate that terraswap pair address is at least valid */
    let pair_contract = maybe_addr(
        deps.api,
        Some(msg.terraswap_pair_addr)
    )?.unwrap();

    /* Validate that leveraged asset address is at least valid */
    let leveraged_asset_contract = maybe_addr(
        deps.api,
        Some(msg.leveraged_asset_addr)
    )?.unwrap();

    /* Read in & validate hyperparameters */
    let hyper_p = Hyperparameters {
        leverage_amount:msg.leverage_amount,
        minimum_protocol_ratio: msg.minimum_protocol_ratio,
        rebalance_ratio: msg.rebalance_ratio,
        mint_premium: msg.mint_premium,
        rebalance_premium: msg.rebalance_premium,
        terraswap_pair_addr: pair_contract,
        leveraged_asset_addr: leveraged_asset_contract,
    };

    /* Save for our reference across contract lifetime */
    HYPERPARAMETERS.save(deps.storage, &hyper_p)?;

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
 * Query entrypoint
 */
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Hyperparameters { } => to_binary(&query_hyperparameters(deps)?),
    }
}

