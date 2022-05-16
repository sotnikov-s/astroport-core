use crate::contract::{execute, instantiate, query};
use crate::error::ContractError::Unauthorized;
use astroport::asset::AssetInfo;
use astroport::fixed_rate_provider::{
    ConfigResponse, ExecuteMsg::UpdateExchangeRate, InstantiateMsg, QueryMsg,
};
use astroport::rate_provider::ExchangeRateResponse;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, Addr, Decimal, Fraction, StdError};

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);
    let asset_0 = AssetInfo::NativeToken {
        denom: "uusd".to_string(),
    };
    let asset_1 = AssetInfo::Token {
        contract_addr: Addr::unchecked("asset0000"),
    };
    let msg = InstantiateMsg {
        asset_infos: [asset_0.clone(), asset_1.clone()],
        exchange_rate: Decimal::from_ratio(1u128, 5u128),
    };
    let info = mock_info("creator", &[]);

    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // check if exchange rate is as set in the init msg
    let er: ExchangeRateResponse = from_binary(
        &query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::ExchangeRate {
                offer_asset: asset_0.clone(),
                ask_asset: asset_1.clone(),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(er.exchange_rate, Decimal::from_ratio(1u128, 5u128));

    // check if config is as set in the init msg
    let config: ConfigResponse =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!(config.exchange_rate, Decimal::from_ratio(1u128, 5u128));
    assert_eq!(config.asset_infos, [asset_0, asset_1]);
}

#[test]
fn query_exchange_rate() {
    let mut deps = mock_dependencies(&[]);
    let asset_0 = AssetInfo::NativeToken {
        denom: "uusd".to_string(),
    };
    let asset_1 = AssetInfo::Token {
        contract_addr: Addr::unchecked("asset0000"),
    };
    let exchange_rate = Decimal::from_ratio(1u128, 5u128);
    let msg = InstantiateMsg {
        asset_infos: [asset_0.clone(), asset_1.clone()],
        exchange_rate,
    };
    let info = mock_info("creator", &[]);

    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // check exchange rate from asset_0 to asset_1, should be equal to the exchange_rate variable
    let er: ExchangeRateResponse = from_binary(
        &query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::ExchangeRate {
                offer_asset: asset_0.clone(),
                ask_asset: asset_1.clone(),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(er.exchange_rate, exchange_rate);

    // check exchange rate from asset_1 to asset_0, should be equal to the exchange_rate.inv()
    let er: ExchangeRateResponse = from_binary(
        &query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::ExchangeRate {
                offer_asset: asset_1.clone(),
                ask_asset: asset_0.clone(),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(er.exchange_rate, exchange_rate.inv().unwrap());

    // check that there is an error response on wrong assets query
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::ExchangeRate {
            offer_asset: asset_1.clone(),
            ask_asset: AssetInfo::Token {
                contract_addr: Addr::unchecked("asset0001"),
            },
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        StdError::generic_err("Given assets don't belong to the pair",)
    );
}

#[test]
fn update_exchange_rate() {
    let mut deps = mock_dependencies(&[]);
    let asset_0 = AssetInfo::NativeToken {
        denom: "uusd".to_string(),
    };
    let asset_1 = AssetInfo::Token {
        contract_addr: Addr::unchecked("asset0000"),
    };
    let msg = InstantiateMsg {
        asset_infos: [asset_0.clone(), asset_1.clone()],
        exchange_rate: Decimal::from_ratio(1u128, 5u128),
    };
    let info = mock_info("creator", &[]);

    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    assert_eq!(0, res.messages.len());

    let er: ExchangeRateResponse = from_binary(
        &query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::ExchangeRate {
                offer_asset: asset_0.clone(),
                ask_asset: asset_1.clone(),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(er.exchange_rate, Decimal::from_ratio(1u128, 5u128));

    // update exchange rate and check if the corresponding query returns new value
    let msg = UpdateExchangeRate {
        exchange_rate: Decimal::from_ratio(2u128, 5u128),
    };
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let er: ExchangeRateResponse = from_binary(
        &query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::ExchangeRate {
                offer_asset: asset_0.clone(),
                ask_asset: asset_1.clone(),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(er.exchange_rate, Decimal::from_ratio(2u128, 5u128));

    // update exchange rate queries from addresses different from creator's address should result in an error
    let info = mock_info("user", &[]);
    let msg = UpdateExchangeRate {
        exchange_rate: Decimal::from_ratio(3u128, 5u128),
    };
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(res, Unauthorized {});

    let er: ExchangeRateResponse = from_binary(
        &query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::ExchangeRate {
                offer_asset: asset_0.clone(),
                ask_asset: asset_1.clone(),
            },
        )
        .unwrap(),
    )
    .unwrap();
    // exchange rate should not change
    assert_eq!(er.exchange_rate, Decimal::from_ratio(2u128, 5u128));
}
