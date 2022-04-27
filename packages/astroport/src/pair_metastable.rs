use cosmwasm_std::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ## Description
/// This structure holds metastableswap pool parameters.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MetastablePoolParams {
    /// The current metastableswap pool amplification
    pub amp: u64,
    /// The exchange rate provider contract address
    pub er_provider_addr: String,
    /// The amount of blocks after that cached exchange rate expires
    pub er_cache_btl: u64,
}

/// ## Description
/// This structure stores a metastableswap pool's configuration.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MetastablePoolConfig {
    /// The metastableswap pool amplification
    pub amp: Decimal,
    /// The exchange rate provider address
    pub er_provider_addr: String,
    /// The amount of blocks after that the exchange rate expires
    pub er_cache_btl: u64,
}

/// ## Description
/// This enum stores the options available to update metastableswap pool parameters.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MetastablePoolUpdateParams {
    StartChangingAmp { next_amp: u64, next_amp_time: u64 },
    StopChangingAmp {},
    UpdateRateProvider { address: String },
    UpdateErCacheBTL { btl: u64 },
}
