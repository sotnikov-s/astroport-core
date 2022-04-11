## A list of things required to be implemented for the metastableswap pool:
NOTE: this is most likely not the final version of the things to do.

Flow:
- [x] introduce exchange rate provider query protocol
- [x] incorporate exchange rate provider abstraction to the pool contract
- [x] add exchange rate cache to the pool contract
- [x] exchange rate calculation in both directions (asset0->asset1 and asset1->asset0) based on a single cached exchange rate value
- [x] incorporate exchange rate into provide_liquidity flow __(no changes were required)__
- [x] incorporate exchange rate into withdraw_liquidity flow __(no changes were required)__
- [x] appropriate slippage calculation (not the zero one like in an ordinary stableswap pool, but the one based on accrued asset amount deviation from offer_asset modified by the exchange_rate) __(no changes were required)__
- [x] implement mock for rate provider to be used in tests
- [x] enhance pool factory to be capable of instantiating pairs of metastableswap type
- [ ] adapt documentation
- [ ] clean up package (remove stableswap pool copy leftovers, cover code with comments)
- [ ] make sure we stick with astroport code, naming and comments patterns

Tests:
- [x] a swap of a significant (compared to pool size) amount of assets is performed with barely notable slippage. in both directions
- [x] simulation queries are performed with barely notable slippage and take into account the exchange rate. in both directions
- [x] query_share provides the amount of assets in accordance with the pool exchange rate
- [x] provide_liquidity's slippage tolerance is applied taking into account the exchange rate __(no changes were required)__
- [x] withdraw_liquidity returns the amount of assets proportional to the exchange rate __(no changes were required)__
- [x] update exchange rate provider address
- [x] update exchange rate cache blocks to live parameter
