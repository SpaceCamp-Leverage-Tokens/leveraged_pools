/*
 * Liquidity manager
 *
 * Provides liquidity deposits and withdrawals
 */
use cosmwasm_std::{
    Uint128, DepsMut, MessageInfo,
};

use crate::error::ContractError;

pub struct ProviderPosition {
    pub asset_pool_partial_share: Uint128,
    pub asset_pool_total_share: Uint128,
}

pub fn execute_provide_liquidity(
    _deps: DepsMut,
    _info: MessageInfo,
) -> Result<ProviderPosition, ContractError> {
    Err(ContractError::Unimplemented{ })
}

pub fn execute_withdraw_liquidity(
    _deps: DepsMut,
    _info: MessageInfo,
) -> Result<ProviderPosition, ContractError> {
    Err(ContractError::Unimplemented{ })
}
