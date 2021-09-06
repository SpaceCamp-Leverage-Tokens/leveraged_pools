/*
 * Minting manager
 *
 * Provides tools for minting and burning a user's leveraged assets
 */

use cosmwasm_std::{
    DepsMut, MessageInfo, Env
};
use crate::leverage_man;
use leveraged_pools::pool::{TryMint, MinterPosition};

use crate::error::ContractError;

pub fn execute_mint_leveraged(
    deps: DepsMut,
    _info: MessageInfo,
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
) -> Result<MinterPosition, ContractError> {
    Err(ContractError::Unimplemented{ })
}
