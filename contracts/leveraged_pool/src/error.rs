use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unimplemented")]
    Unimplemented {},

    /* TODO This is StdError::SerializeErr */
    #[error("Unable to serialize response")]
    SerializeErr { },

    #[error("Unexpected oracle response")]
    UnexpectedOracleResponse {},

    #[error("Token has no liquidity")]
    NoTokenLiquidity {},

    #[error("Passed address was invalid")]
    InvalidAddr {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid Leveraged Pool Params")]
    InvalidPoolParams {},

    #[error("Insufficient Funds")]
    InsuficientFunds {},
}
