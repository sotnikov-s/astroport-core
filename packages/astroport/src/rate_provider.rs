use crate::asset::AssetInfo;
use cosmwasm_std::{
    to_binary, Addr, Decimal, QuerierWrapper, QueryRequest, StdError, StdResult, WasmQuery,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ## Description
/// This structure describes the query messages available in the contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// ## Description
    /// Retrieves the current exchange rate between assets (i.e. how many ask_assets user will
    /// receive for providing one offer_asset)
    GetExchangeRate {
        offer_asset: AssetInfo,
        ask_asset: AssetInfo,
    },
}

/// ## Description
/// This structure holds the parameters that are returned from a successful get exchange rate query.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetExchangeRateResponse {
    pub offer_asset: AssetInfo,
    pub ask_asset: AssetInfo,
    pub exchange_rate: Decimal,
}

/// ## Description
/// Returns information about an asset's price from a specific pair using a [`SimulationResponse`] object.
/// ## Params
/// * **querier** is an object of type [`QuerierWrapper`].
///
/// * **pair_contract** is an object of type [`Addr`]. This is the pair that holds the target asset.
///
/// * **asset** is an object of type [`Asset`]. This is the asset for which we return the simulated price.
pub fn query_exchange_rate(
    querier: &QuerierWrapper,
    offer_asset: &AssetInfo,
    ask_asset: &AssetInfo,
    rate_provider_contract: Addr,
) -> StdResult<GetExchangeRateResponse> {
    let er: GetExchangeRateResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: rate_provider_contract.to_string(),
        msg: to_binary(&QueryMsg::GetExchangeRate {
            offer_asset: offer_asset.clone(),
            ask_asset: ask_asset.clone(),
        })?,
    }))?;

    if er.exchange_rate <= Decimal::zero() {
        return Err(StdError::generic_err(
            "Exchange rate from rate provider must be greater that zero",
        ));
    }
    Ok(er)
}
