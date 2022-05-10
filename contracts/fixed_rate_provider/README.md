# Astroport Fixed Rate Provider

Fixed rate provider is used in pools of the metastableswap type for supplying a constant exchange rate between assets in the pool. Although the exchange rate in the provider is constant, the rate provider contract creator is allowed to manually update it, making it constant in a timespan.

Fixed rate provider is just an option to be used as a rate provider, and most likely just an example and test one. Metaswableswap pools may (and most probably will) use custom variations of rate providers with more complicated and refined logic of exchange rate determination.

---

## InstantiateMsg

Initializes a new fixed rate provider.

```json
{
  "asset_infos": [
    {
      "token": {
        "contract_addr": "terra..."
      }
    },
    {
      "native_token": {
        "denom": "uusd"
      }
    }
  ],
  "exchange_rate": "1.3"
}
```

## ExecuteMsg

### update_exchange_rate

Updates the provided exchange rate.

```json
{
  "update_exchange_rate": {
    "exchange_rate": "0.7847",
  }
}
```

## QueryMsg

All query messages are described below. A custom struct is defined for each query response.

### exchange_rate

Returns the set exchange rate between assets, i.e. how many ask_assets user will receive for providing one offer_asset.

```json
{
  "exchange_rate": {
    "offer_asset": {
      "token": {
        "contract_addr": "terra..."
      }
    },
    "ask_asset": {
      "native_token": {
        "denom": "uusd"
      }
    }
  }
}
```

### config

Get the rate provider contract configuration.

```json
{
  "config": {}
}
```
