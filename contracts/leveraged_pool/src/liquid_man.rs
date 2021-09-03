/*
 * Liquidity manager
 *
 * Provides liquidity deposits and withdrawals
 */
use cosmwasm_std::{
    Uint128, DepsMut, MessageInfo, //Env, Storage, Api, QuerierWrapper, Addr
};
use crate::error::ContractError;
// use cw_storage_plus::{Item, Map};
// use leveraged_pools::pool::{InstantiateMsg, PriceSnapshot};
// use crate::{leverage_man};
// use crate::swap::TSLiason;


pub struct ProviderPosition {
    pub asset_pool_partial_share: Uint128,
    pub asset_pool_total_share: Uint128,
}

pub fn execute_provide_liquidity(
    _deps: DepsMut,
    _info: MessageInfo,
) -> Result<ProviderPosition, ContractError> {
    // let my_position = leverage_man::get_liquidity_map(&deps,info.sender);
    
    // Get total asset share form leverage_man?
    // Get mapping from Addr to asset_partial_share from leverage_man

    //Accept funds from user
    //Update total asset share 
    //Update partial share mapping 

    Err(ContractError::Unimplemented{ })
}

pub fn execute_withdraw_liquidity(
    _deps: DepsMut,
    _info: MessageInfo,
) -> Result<ProviderPosition, ContractError> {
    Err(ContractError::Unimplemented{ })
}


