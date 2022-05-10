use cosmwasm_std::StdError;
use thiserror::Error;

/// ## Description
/// This enum describes stableswap pair contract errors!
#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Doubling assets in asset infos")]
    DoublingAssets {},

    #[error("Exchange rate must be greater that zero")]
    InvalidExchangeRate {},
}
