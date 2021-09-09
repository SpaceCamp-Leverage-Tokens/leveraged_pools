use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Error in computing arithmetic result")]
    /* TODO This is DivideByZero and Overflow and friends */
    ArithmeticError {},

    #[error("Unimplemented")]
    Unimplemented {},

    #[error("Total minted value is zero")]
    NoMintedValue {},

    /* TODO This is StdError::SerializeErr */
    #[error("Unable to serialize response")]
    SerializeErr {},

    #[error("Received asset does not match pool denomination")]
    WrongAssetLOL {},

    #[error("Unexpected oracle response")]
    UnexpectedOracleResponse {},

    #[error("Proposed transaction would destabilize the pool")]
    WouldViolatePoolHealth {},

    #[error("Token has no liquidity")]
    NoTokenLiquidity {},

    #[error("Passed address was invalid")]
    InvalidAddr {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid Leveraged Pool Params")]
    InvalidPoolParams {},

    #[error("Insufficient Funds")]
    InsufficientFunds {},

    #[error(
        "Action taken will lower Protocol Ratio lower than Rebalance Ratio"
    )]
    InvalidPoolState {},

    #[error("Insufficient Funds")]
    Generic {},
}
