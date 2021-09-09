/*
 * Minting manager
 *
 * Provides tools for minting and burning a user's leveraged assets
 */

use crate::leverage_man;
use cosmwasm_std::{
    to_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use leveraged_pools::pool::{MinterPosition, TryBurn, TryMint};

use crate::error::ContractError;

/**
 * Validate and mint the `proposed_mint` position
 */
pub fn execute_mint_leveraged(
    deps: DepsMut,
    _info: &MessageInfo,
    env: &Env,
    proposed_mint: &TryMint,
) -> Result<MinterPosition, ContractError> {
    let state = leverage_man::query_pool_state(&deps.as_ref())?;
    let hyper_p = leverage_man::query_hyperparameters(&deps.as_ref())?;

    /* The unleveraged funds that were sent in the mint tx */
    let sent_unleveraged_assets = proposed_mint.amount;

    /* How many leveraged assets could these unleveraged assets buy */
    let new_leveraged_assets = leverage_man::leveraged_equivalence(
        &deps.as_ref(),
        env,
        sent_unleveraged_assets,
    )?;

    /*
     * For deposits, we include the sent funds in the PR calculation
     * (AIR + sent_funds) / (leveraged_assets + equivalence(sent_funds)) >= PR
     */
    if leverage_man::calculate_pr(
        &deps.as_ref(),
        env,
        state.assets_in_reserve + sent_unleveraged_assets,
        state.total_leveraged_pool_share + new_leveraged_assets,
    )? < hyper_p.minimum_protocol_ratio
    {
        return Err(ContractError::WouldViolatePoolHealth {});
    }

    leverage_man::create_leveraged_position(
        deps.storage,
        &proposed_mint.sender,
        new_leveraged_assets,
        sent_unleveraged_assets,
    )
}

/**
 * Validate and burn the `proposed_burn` leveraged position
 */
pub fn execute_burn_leveraged(
    deps: DepsMut,
    _info: &MessageInfo,
    env: &Env,
    proposed_burn: &TryBurn,
) -> Result<Response, ContractError> {
    let state = leverage_man::query_pool_state(&deps.as_ref())?;
    let hyper_p = leverage_man::query_hyperparameters(&deps.as_ref())?;
    let leverage_share = leverage_man::get_addr_leveraged_share(
        &deps.as_ref(),
        &proposed_burn.sender,
    );

    let proposed_share = proposed_burn.pool_share;

    if !leverage_man::addr_has_adequate_leveraged_share(
        &deps.as_ref(),
        &proposed_burn.sender,
        proposed_burn.pool_share,
    ) {
        Err(ContractError::InsufficientFunds {})?;
    }

    let proposed_burn_units = leverage_share
        .multiply_ratio(proposed_share, state.total_leveraged_pool_share);

    let proposed_redeem_units = leverage_man::unleveraged_equivalence(
        &deps.as_ref(),
        env,
        proposed_burn_units,
    )?;

    if leverage_man::calculate_pr(
        &deps.as_ref(),
        env,
        state.assets_in_reserve - proposed_redeem_units,
        state.total_leveraged_pool_share - proposed_burn_units,
    )? < hyper_p.minimum_protocol_ratio
    {
        return Err(ContractError::WouldViolatePoolHealth {});
    }

    leverage_man::burn_leveraged_position(
        deps.storage,
        &proposed_burn.sender,
        proposed_burn_units,
        proposed_redeem_units,
    )?;

    /* TODO this is inappropriate here, should be in
     * contract.rs, but I need the leveraged_asset_addr.
     * Maybe could pass that back through MinterPosition */
    let burn_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps
            .api
            .addr_humanize(&hyper_p.leveraged_asset_addr)?
            .to_string(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: proposed_burn.sender.to_string(),
            amount: proposed_redeem_units,
        })?,
    });

    Ok(Response::new().add_message(burn_msg))
}
