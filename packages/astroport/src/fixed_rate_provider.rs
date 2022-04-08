use crate::asset::AssetInfo;
use cosmwasm_std::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ## Description
/// This structure describes the parameters used for creating a contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Information about the two assets in the pool
    pub asset_infos: [AssetInfo; 2],
    /// The rate of exchange of asset_0 to asset_1
    pub exchange_rate: Decimal,
}

/// ## Description
/// This structure describes the execute messages available in the contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Update the pair exchange rate
    UpdateExchangeRate { exchange_rate: Decimal },
}

/// ## Description
/// This structure describes the query messages available in the contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// ## Description
    /// Retrieves the current exchange rate between assets in a [`rate_provider::GetExchangeRateResponse`] structure.
    GetExchangeRate {
        offer_asset: AssetInfo,
        ask_asset: AssetInfo,
    },
    /// Returns contract configuration settings in a custom [`ConfigResponse`] structure.
    Config {},
}

/// ## Description
/// This struct is used to return a query result with the general contract configuration.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    /// Information about the two assets in the pool
    pub asset_infos: [AssetInfo; 2],
    /// The rate of exchange of asset_0 to asset_1
    pub exchange_rate: Decimal,
}
