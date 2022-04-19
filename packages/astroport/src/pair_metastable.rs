use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::asset::Asset;

use cosmwasm_std::{Binary, Decimal, Uint128};
use cw20::Cw20ReceiveMsg;

/// ## Description
/// This structure describes the execute messages available in the contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// ## Description
    /// Receives a message of type [`Cw20ReceiveMsg`]
    Receive(Cw20ReceiveMsg),
    /// ProvideLiquidity allows someone to provide liquidity in the pool
    ProvideLiquidity {
        /// The assets available in the pool
        assets: [Asset; 2],
        /// The slippage tolerance that allows liquidity provision only if the price in the pool doesn't move too much
        slippage_tolerance: Option<Decimal>,
        /// Determines whether the LP tokens minted for the user is auto_staked in the Generator contract
        auto_stake: Option<bool>,
        /// The receiver of LP tokens
        receiver: Option<String>,
    },
    /// Swap performs a swap in the pool
    Swap {
        offer_asset: Asset,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
        to: Option<String>,
    },
    /// Update the pair configuration
    UpdateConfig { params: Binary },
}

/// ## Description
/// This structure describes a CW20 hook message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    /// Swap a given amount of asset
    Swap {
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
        to: Option<String>,
    },
    /// Withdraw liquidity from the pool
    WithdrawLiquidity {},
}

/// ## Description
/// This structure describes the query messages available in the contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns information about a pair in an object of type [`super::asset::PairInfo`].
    Pair {},
    /// Returns information about a pool in an object of type [`PoolResponse`].
    Pool {},
    /// Returns contract configuration settings in a custom [`ConfigResponse`] structure.
    Config {},
    /// Returns information about the share of the pool in a vector that contains objects of type [`Asset`].
    Share { amount: Uint128 },
    /// Returns information about a swap simulation in a [`SimulationResponse`] object.
    Simulation { offer_asset: Asset },
    /// Returns information about cumulative prices in a [`CumulativePricesResponse`] object.
    ReverseSimulation { ask_asset: Asset },
    /// Returns information about the cumulative prices in a [`CumulativePricesResponse`] object
    CumulativePrices {},
}

/// ## Description
/// This struct is used to return a query result with the total amount of LP tokens and the two assets in a specific pool.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PoolResponse {
    /// The assets in the pool together with asset amounts
    pub assets: [Asset; 2],
    /// The total amount of LP tokens currently issued
    pub total_share: Uint128,
}

/// ## Description
/// This struct is used to return a query result with the general contract configuration.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    /// Last timestamp when the cumulative prices in the pool were updated
    pub block_time_last: u64,
    /// The pool's parameters
    pub params: Option<Binary>,
}

/// ## Description
/// This structure holds the parameters that are returned from a swap simulation response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SimulationResponse {
    /// The amount of ask assets returned by the swap
    pub return_amount: Uint128,
    /// The spread used in the swap operation
    pub spread_amount: Uint128,
    /// The amount of fees charged by the transaction
    pub commission_amount: Uint128,
}

/// ## Description
/// This structure holds the parameters that are returned from a reverse swap simulation response.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ReverseSimulationResponse {
    /// The amount of offer assets returned by the reverse swap
    pub offer_amount: Uint128,
    /// The spread used in the swap operation
    pub spread_amount: Uint128,
    /// The amount of fees charged by the transaction
    pub commission_amount: Uint128,
}

/// ## Description
/// This structure is used to return a cumulative prices query response.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CumulativePricesResponse {
    /// The two assets in the pool to query
    pub assets: [Asset; 2],
    /// The total amount of LP tokens currently issued
    pub total_share: Uint128,
    /// The last value for the token0 cumulative price
    pub price0_cumulative_last: Uint128,
    /// The last value for the token1 cumulative price
    pub price1_cumulative_last: Uint128,
}

/// ## Description
/// This structure describes a migration message.
/// We currently take no arguments for migrations.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

/// ## Description
/// This structure holds metastableswap pool parameters.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MetaStablePoolParams {
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
pub struct MetaStablePoolConfig {
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
pub enum MetaStablePoolUpdateParams {
    StartChangingAmp { next_amp: u64, next_amp_time: u64 },
    StopChangingAmp {},
    UpdateRateProvider { address: String },
    UpdateErCacheBTL { btl: u64 },
}
