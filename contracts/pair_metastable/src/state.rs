use astroport::asset::{AssetInfo, PairInfo};
use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{Addr, Decimal, StdError, StdResult, Uint128};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::ops::Add;

/// ## Description
/// This structure stores the main stableswap pair parameters.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// The pair information stored in a [`PairInfo`] struct
    pub pair_info: PairInfo,
    /// The factory contract address
    pub factory_addr: Addr,
    /// The last timestamp when the pair contract update the asset cumulative prices
    pub block_time_last: u64,
    /// The last cumulative price for asset 0
    pub price0_cumulative_last: Uint128,
    /// The last cumulative price for asset 1
    pub price1_cumulative_last: Uint128,
    /// The exchange rate provider address
    pub er_provider_addr: Addr,
    // This is the current amplification used in the pool
    pub init_amp: u64,
    // This is the start time when amplification starts to scale up or down
    pub init_amp_time: u64,
    // This is the target amplification to reach at `next_amp_time`
    pub next_amp: u64,
    // This is the timestamp when the current pool amplification should be `next_amp`
    pub next_amp_time: u64,
}

/// ## Description
/// This structure stores temporary exchange rate information for a pair.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TmpPairExchangeRate {
    /// Asset information for the assets in the pair
    pub asset_infos: [AssetInfo; 2],
    /// The proportion in exchange of asset 0 to asset 1
    pub exchange_rate: Decimal,
    /// The blockchain height of the exchange rate update
    pub height: u64,
    /// The amount of blocks after that the exchange rate expires
    pub btl: u64,
}

impl TmpPairExchangeRate {
    pub fn new(asset_infos: [AssetInfo; 2], btl: u64) -> TmpPairExchangeRate {
        TmpPairExchangeRate {
            asset_infos,
            btl,
            exchange_rate: Decimal::zero(),
            height: 0u64,
        }
    }

    /// ## Description
    /// Returns the exchange rate between assets
    pub fn get_rate(&self, asset_infos: [AssetInfo; 2]) -> StdResult<Decimal> {
        if asset_infos[0].equal(&self.asset_infos[0]) && asset_infos[1].equal(&self.asset_infos[1])
        {
            return Ok(self.exchange_rate);
        } else if asset_infos[0].equal(&self.asset_infos[1])
            && asset_infos[1].equal(&self.asset_infos[0])
        {
            return Ok((Decimal256::one() / Decimal256::from(self.exchange_rate)).into());
        }
        return Err(StdError::generic_err(
            "Given assets don't belong to the pair",
        ));
    }

    /// ## Description
    /// Updates the proportion in exchange of asset 0 to asset 1
    pub fn update_rate(&mut self, exchange_rate: Decimal, height: u64) {
        self.exchange_rate = exchange_rate;
        self.height = height;
    }

    /// ## Description
    /// Updates the cached exchange rate time to live measured in blocks
    pub fn update_btl(&mut self, btl: u64) {
        self.btl = btl;
    }

    pub fn is_expired(&self, height: u64) -> bool {
        height.ge(&self.height.add(self.btl))
    }
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const ER_CACHE: Item<TmpPairExchangeRate> = Item::new("er_cache");
