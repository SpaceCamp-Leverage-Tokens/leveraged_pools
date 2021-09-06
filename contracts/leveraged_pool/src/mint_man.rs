/*
 * Minting manager
 *
 * Provides tools for minting and burning a user's leveraged assets
 */

use cosmwasm_std::{
    DepsMut, MessageInfo, Env
};
use crate::leverage_man;
use leveraged_pools::pool::{TryMint, MinterPosition, TryBurn};

use crate::error::ContractError;

pub fn execute_mint_leveraged(
    deps: DepsMut,
    _info: &MessageInfo,
    _env: &Env,
    proposed_mint: &TryMint,
) -> Result<MinterPosition, ContractError> {
    let state = leverage_man::query_pool_state(&deps.as_ref())?;
    let hyper_p = leverage_man::query_hyperparameters(&deps.as_ref())?;

    let unleveraged_assets = proposed_mint.amount;
    let leveraged_assets = leverage_man::leveraged_equivalence(
            &deps.as_ref(), unleveraged_assets,
    );

    if leverage_man::calculate_pr(
        state.assets_in_reserve,
        state.total_leveraged_assets + leveraged_assets,
    ) < hyper_p.minimum_protocol_ratio {
        return Err(ContractError::WouldViolatePoolHealth{ });
    }

    leverage_man::create_leveraged_position(
        deps.storage,
        &proposed_mint.sender,
        leveraged_assets,
        unleveraged_assets,
    )
}

pub fn execute_burn_leveraged(
    deps: DepsMut,
    _info: &MessageInfo,
    _env: &Env,
    proposed_burn: &TryBurn,
) -> Result<MinterPosition, ContractError> {
    let state = leverage_man::query_pool_state(&deps.as_ref())?;
    let hyper_p = leverage_man::query_hyperparameters(&deps.as_ref())?;

    let proposed_share = proposed_burn.pool_share;

    if !leverage_man::addr_has_adequate_leveraged_share(
        &deps.as_ref(),
        &proposed_burn.sender,
        proposed_burn.pool_share
    ) {
        Err(ContractError::InsufficientFunds{ })?;
    }

    let proposed_burn_units = proposed_share.checked_div(
        state.total_leveraged_pool_share).or_else(
            |_| Err(ContractError::ArithmeticError{ }))?.checked_mul(
                state.total_leveraged_assets).or_else(
                    |_| Err(ContractError::ArithmeticError{ }))?;

    let proposed_redeem_units = leverage_man::unleveraged_equivalence(
        &deps.as_ref(),
        proposed_burn_units,
    );

    if leverage_man::calculate_pr(
        state.assets_in_reserve - proposed_redeem_units,
        state.total_leveraged_assets - proposed_burn_units,
    ) < hyper_p.minimum_protocol_ratio {
        return Err(ContractError::WouldViolatePoolHealth{ });
    }

    leverage_man::burn_leveraged_position(
        deps.storage,
        &proposed_burn.sender,
        proposed_burn_units,
        proposed_redeem_units,
    )
}
