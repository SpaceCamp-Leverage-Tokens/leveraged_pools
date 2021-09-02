pub mod contract;
mod error;
pub mod msg;
pub mod state;
pub mod swap;

/* MAGI System */
pub mod leverage_man;
pub mod mint_man;
pub mod liquid_man;

#[cfg(test)]
mod testing;

pub use crate::error::ContractError;
