use astroport::asset::{AssetInfo, PairInfo};
use cosmwasm_std::{Addr, Decimal, Fraction, StdError, StdResult, Uint128};
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
pub struct CachedExchangeRate {
    /// Asset information for the assets in the pair
    asset_infos: [AssetInfo; 2],
    /// The proportion in exchange of asset 0 to asset 1
    exchange_rate: Decimal,
    /// The blockchain height of the exchange rate update
    height: u64,
    /// The amount of blocks after that the exchange rate expires
    btl: u64,
}

impl CachedExchangeRate {
    pub fn new(
        asset_infos: [AssetInfo; 2],
        exchange_rate: Decimal,
        height: u64,
        btl: u64,
    ) -> StdResult<Self> {
        if exchange_rate <= Decimal::zero() {
            return Err(StdError::generic_err(
                "Exchange rate must be greater that zero",
            ));
        }
        if btl == 0 {
            return Err(StdError::generic_err(
                "Exchange rate cache blocks to live must be greater than 0",
            ));
        }

        Ok(CachedExchangeRate {
            asset_infos,
            btl,
            exchange_rate,
            height,
        })
    }

    /// ## Description
    /// Returns the assets pair
    pub fn get_assets(&self) -> [AssetInfo; 2] {
        [self.asset_infos[0].clone(), self.asset_infos[1].clone()]
    }

    /// ## Description
    /// Returns the exchange rate between assets
    pub fn get_rate(&self, asset_infos: [&AssetInfo; 2]) -> StdResult<Decimal> {
        if asset_infos[0].equal(&self.asset_infos[0]) && asset_infos[1].equal(&self.asset_infos[1])
        {
            return Ok(self.exchange_rate);
        } else if asset_infos[0].equal(&self.asset_infos[1])
            && asset_infos[1].equal(&self.asset_infos[0])
        {
            return Ok(self.exchange_rate.inv().unwrap());
        }
        return Err(StdError::generic_err(
            "Given assets don't belong to the pair",
        ));
    }

    /// ## Description
    /// Updates the proportion in exchange of asset 0 to asset 1
    pub fn update_rate(
        &mut self,
        asset_infos: [&AssetInfo; 2],
        exchange_rate: Decimal,
        height: u64,
    ) -> StdResult<()> {
        if exchange_rate <= Decimal::zero() {
            return Err(StdError::generic_err(
                "Exchange rate must be greater that zero",
            ));
        }

        if asset_infos[0].equal(&self.asset_infos[0]) && asset_infos[1].equal(&self.asset_infos[1])
        {
            self.exchange_rate = exchange_rate;
            self.height = height;
            return Ok(());
        } else if asset_infos[0].equal(&self.asset_infos[1])
            && asset_infos[1].equal(&self.asset_infos[0])
        {
            self.exchange_rate = exchange_rate.inv().unwrap();
            self.height = height;
            return Ok(());
        }
        return Err(StdError::generic_err(
            "Given assets don't belong to the pair",
        ));
    }

    /// ## Description
    /// Updates the cached exchange rate time to live measured in blocks
    pub fn update_btl(&mut self, btl: u64) -> StdResult<()> {
        if btl == 0 {
            return Err(StdError::generic_err(
                "Exchange rate cache blocks to live must be greater than 0",
            ));
        }

        self.btl = btl;
        Ok(())
    }

    /// ## Description
    /// Returns the cached value lifetime measured in blocks
    pub fn get_btl(&self) -> u64 {
        self.btl
    }

    /// ## Description
    /// Returns whether the cached value has expired
    pub fn is_expired(&self, height: u64) -> bool {
        height.ge(&self.height.add(self.btl))
    }
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const ER_CACHE: Item<CachedExchangeRate> = Item::new("er_cache");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tmp_pair_exchange_rate() {
        let asset_0 = AssetInfo::NativeToken {
            denom: String::from("uusd"),
        };
        let asset_1 = AssetInfo::Token {
            contract_addr: Addr::unchecked("asset0000"),
        };

        // check proper initialization
        let mut er = CachedExchangeRate::new(
            [asset_0.clone(), asset_1.clone()],
            Decimal::from_ratio(1u128, 5u128),
            1u64,
            10u64,
        )
        .unwrap();
        assert_eq!(er.asset_infos[0], asset_0);
        assert_eq!(er.asset_infos[1], asset_1);
        assert_eq!(er.btl, 10);
        assert_eq!(er.height, 1);
        assert_eq!(er.exchange_rate, Decimal::from_ratio(1u128, 5u128));

        // check cached value expiration
        assert_eq!(er.is_expired(1), false);
        assert_eq!(er.is_expired(11), true);

        // update btl and check expiration
        er.update_btl(20).unwrap();
        assert_eq!(er.is_expired(1), false);
        assert_eq!(er.is_expired(11), false);
        assert_eq!(er.is_expired(21), true);

        // check get exchange rate in both directions
        assert_eq!(
            er.get_rate([&asset_0, &asset_1]),
            Ok(Decimal::from_ratio(1u128, 5u128))
        );
        assert_eq!(
            er.get_rate([&asset_1, &asset_0]),
            Ok(Decimal::from_ratio(5u128, 1u128))
        );

        // make sure an error response is returned in token mismatch cases
        let asset_2 = AssetInfo::NativeToken {
            denom: String::from("uluna"),
        };
        assert_eq!(
            er.get_rate([&asset_0, &asset_2]),
            Err(StdError::generic_err(
                "Given assets don't belong to the pair",
            ))
        );
        assert_eq!(
            er.get_rate([&asset_1, &asset_2]),
            Err(StdError::generic_err(
                "Given assets don't belong to the pair",
            ))
        );
        assert_eq!(
            er.get_rate([&asset_0, &asset_0]),
            Err(StdError::generic_err(
                "Given assets don't belong to the pair",
            ))
        );
        assert_eq!(
            er.update_rate([&asset_1, &asset_2], Decimal::from_ratio(2u128, 1u128), 1),
            Err(StdError::generic_err(
                "Given assets don't belong to the pair",
            ))
        );

        // check update rate
        assert_eq!(
            er.update_rate([&asset_0, &asset_1], Decimal::from_ratio(2u128, 5u128), 1),
            Ok(())
        );
        assert_eq!(
            er.get_rate([&asset_0, &asset_1]),
            Ok(Decimal::from_ratio(2u128, 5u128))
        );
        assert_eq!(
            er.get_rate([&asset_1, &asset_0]),
            Ok(Decimal::from_ratio(5u128, 2u128))
        );

        // check update rate with zero
        assert_eq!(
            er.update_rate([&asset_0, &asset_1], Decimal::zero(), 1),
            Err(StdError::generic_err(
                "Exchange rate must be greater that zero",
            ))
        );
    }
}
