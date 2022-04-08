use crate::error::ContractError;
use crate::state::{Config, CONFIG};
use astroport::asset::AssetInfo;
use astroport::fixed_rate_provider::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use astroport::rate_provider::GetExchangeRateResponse;
use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{
    entry_point, to_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult,
};

/// ## Description
/// Creates a new contract with the specified parameters in [`InstantiateMsg`].
/// Returns a [`Response`] with the specified attributes if the operation was successful,
/// or a [`ContractError`] if the contract was not created.
/// ## Params
/// * **deps** is an object of type [`DepsMut`].
///
/// * **env** is an object of type [`Env`].
///
/// * **_info** is an object of type [`MessageInfo`].
/// * **msg** is a message of type [`InstantiateMsg`] which contains the parameters for creating the contract.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    msg.asset_infos[0].check(deps.api)?;
    msg.asset_infos[1].check(deps.api)?;

    if msg.asset_infos[0] == msg.asset_infos[1] {
        return Err(ContractError::DoublingAssets {});
    }

    let config = Config {
        asset_infos: msg.asset_infos,
        exchange_rate: msg.exchange_rate,
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new())
}

/// ## Description
/// Exposes all the execute functions available in the contract.
/// ## Params
/// * **deps** is an object of type [`Deps`].
///
/// * **env** is an object of type [`Env`].
///
/// * **info** is an object of type [`MessageInfo`].
///
/// * **msg** is an object of type [`ExecuteMsg`].
///
/// ## Queries
/// * **ExecuteMsg::UpdateExchangeRate {
///     exchange_rate,
/// }** Updates the providing exchange rate between assets.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateExchangeRate { exchange_rate } => {
            update_exchange_rate(deps, env, info, exchange_rate)
        }
    }
}

/// ## Description
/// Updates the providing exchange rate between assets.
///
/// ## Params
/// * **deps** is an object of type [`DepsMut`].
///
/// * **_env** is an object of type [`Env`].
///
/// * **_info** is an object of type [`MessageInfo`].
///
/// * **exchange_rate** is an object of type [`Decimal`] that represents the exchange rate between assets.
pub fn update_exchange_rate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    exchange_rate: Decimal,
) -> Result<Response, ContractError> {
    CONFIG.update(deps.storage, |mut prev_state| -> StdResult<_> {
        prev_state.exchange_rate = exchange_rate;
        Ok(prev_state)
    })?;

    Ok(Response::default())
}

/// ## Description
/// Exposes all the queries available in the contract.
/// ## Params
/// * **deps** is an object of type [`Deps`].
///
/// * **_env** is an object of type [`Env`].
///
/// * **msg** is an object of type [`QueryMsg`].
///
/// ## Queries
/// * **QueryMsg::GetExchangeRate {
///     offer_asset,
///     ask_asset,
/// }** Returns information about the pair exchange rate using a custom [`GetExchangeRateResponse`] structure.
///
/// * **QueryMsg::Config {}** Returns general contract parameters using a custom [`ConfigResponse`] structure.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetExchangeRate {
            offer_asset,
            ask_asset,
        } => to_binary(&query_rate(deps, offer_asset, ask_asset)?),
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

/// ## Description
/// Returns information about the pair exchange rate using a custom [`GetExchangeRateResponse`] structure.
/// ## Params
/// * **deps** is an object of type [`Deps`].
///
/// * **offer_asset** is an object of type [`AssetInfo`]. Proposed asset for swapping.
///
/// * **ask_asset** is an object of type [`AssetInfo`] and represents the asset that we swap to.
pub fn query_rate(
    deps: Deps,
    offer_asset: AssetInfo,
    ask_asset: AssetInfo,
) -> StdResult<GetExchangeRateResponse> {
    let config: Config = CONFIG.load(deps.storage)?;

    let exchange_rate = if config.asset_infos[0].equal(&offer_asset)
        && config.asset_infos[1].equal(&ask_asset)
    {
        config.exchange_rate
    } else if config.asset_infos[0].equal(&ask_asset) && config.asset_infos[1].equal(&offer_asset) {
        (Decimal256::one() / Decimal256::from(config.exchange_rate)).into()
    } else {
        return Err(StdError::generic_err(
            "Given ask asset doesn't belong to pairs",
        ));
    };

    let resp = GetExchangeRateResponse {
        offer_asset,
        ask_asset,
        exchange_rate,
    };
    Ok(resp)
}

/// ## Description
/// Returns the pair contract configuration in a [`ConfigResponse`] object.
/// ## Params
/// * **deps** is an object of type [`Deps`].
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        asset_infos: config.asset_infos,
        exchange_rate: config.exchange_rate,
    })
}
