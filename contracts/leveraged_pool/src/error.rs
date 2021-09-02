use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unimplemented")]
    Unimplemented {},

    #[error("Unexpected oracle response")]
    UnexpectedOracleResponse {},

    #[error("Token has no liquidity")]
    NoTokenLiquidity {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid Leveraged Pool Params")]
    InvalidPoolParams {},
}
