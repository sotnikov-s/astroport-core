use astroport::asset::AssetInfo;
use cosmwasm_std::{Addr, Decimal};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CONFIG: Item<Config> = Item::new("config");

/// ## Description
/// This structure stores the fixed rate provider parameters.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// Address of the creator of the rate provider contract
    pub creator: Addr,
    /// Information about the two assets in the related pool
    pub asset_infos: [AssetInfo; 2],
    /// The rate of exchange of asset_0 to asset_1
    pub exchange_rate: Decimal,
}
