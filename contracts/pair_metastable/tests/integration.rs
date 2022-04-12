use astroport::asset::{Asset, AssetInfo, PairInfo};
use astroport::factory::{
    ExecuteMsg as FactoryExecuteMsg, InstantiateMsg as FactoryInstantiateMsg, PairConfig, PairType,
    QueryMsg as FactoryQueryMsg,
};
use astroport::pair::InstantiateMsg;
use astroport::pair::TWAP_PRECISION;
use astroport::pair_metastable::{
    ConfigResponse, CumulativePricesResponse, Cw20HookMsg, ExecuteMsg, MetaStablePoolConfig,
    MetaStablePoolParams, MetaStablePoolUpdateAmp, QueryMsg,
};

use astroport::fixed_rate_provider::{
    InstantiateMsg as RateProviderInstantiateMsg, QueryMsg as RateProviderQueryMsg,
};
use astroport::rate_provider::GetExchangeRateResponse;
use astroport::token::InstantiateMsg as TokenInstantiateMsg;
use astroport_pair_metastable::math::{MAX_AMP, MAX_AMP_CHANGE, MIN_AMP_CHANGING_TIME};
use cosmwasm_std::testing::{mock_env, MockApi, MockStorage};
use cosmwasm_std::{
    attr, from_binary, to_binary, Addr, Coin, Decimal, QueryRequest, Uint128, WasmQuery,
};
use cw20::{BalanceResponse, Cw20Coin, Cw20ExecuteMsg, Cw20QueryMsg, MinterResponse};
use terra_multi_test::{AppBuilder, BankKeeper, ContractWrapper, Executor, TerraApp, TerraMock};

const OWNER: &str = "owner";

fn mock_app() -> TerraApp {
    let env = mock_env();
    let api = MockApi::default();
    let bank = BankKeeper::new();
    let storage = MockStorage::new();
    let custom = TerraMock::luna_ust_case();

    AppBuilder::new()
        .with_api(api)
        .with_block(env.block)
        .with_bank(bank)
        .with_storage(storage)
        .with_custom(custom)
        .build()
}

fn store_token_code(app: &mut TerraApp) -> u64 {
    let astro_token_contract = Box::new(ContractWrapper::new_with_empty(
        astroport_token::contract::execute,
        astroport_token::contract::instantiate,
        astroport_token::contract::query,
    ));

    app.store_code(astro_token_contract)
}

fn store_pair_code(app: &mut TerraApp) -> u64 {
    let pair_contract = Box::new(
        ContractWrapper::new_with_empty(
            astroport_pair_metastable::contract::execute,
            astroport_pair_metastable::contract::instantiate,
            astroport_pair_metastable::contract::query,
        )
        .with_reply_empty(astroport_pair_metastable::contract::reply),
    );

    app.store_code(pair_contract)
}

fn store_rate_provider_code(app: &mut TerraApp) -> u64 {
    let rate_provider_contract = Box::new(
        ContractWrapper::new_with_empty(
            astroport_fixed_rate_provider::contract::execute,
            astroport_fixed_rate_provider::contract::instantiate,
            astroport_fixed_rate_provider::contract::query,
        )
        .with_reply_empty(astroport_pair_metastable::contract::reply),
    );

    app.store_code(rate_provider_contract)
}

fn store_factory_code(app: &mut TerraApp) -> u64 {
    let factory_contract = Box::new(
        ContractWrapper::new_with_empty(
            astroport_factory::contract::execute,
            astroport_factory::contract::instantiate,
            astroport_factory::contract::query,
        )
        .with_reply_empty(astroport_factory::contract::reply),
    );

    app.store_code(factory_contract)
}

fn instantiate_pair(mut router: &mut TerraApp, owner: &Addr) -> Addr {
    let token_contract_code_id = store_token_code(&mut router);
    let pair_contract_code_id = store_pair_code(&mut router);
    let rate_provider_contract_code_id = store_rate_provider_code(&mut router);
    let asset_infos = [
        AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
        AssetInfo::NativeToken {
            denom: "uluna".to_string(),
        },
    ];

    let msg = InstantiateMsg {
        asset_infos: asset_infos.clone(),
        token_code_id: token_contract_code_id,
        factory_addr: String::from("factory"),
        init_params: None,
    };

    let resp = router
        .instantiate_contract(
            pair_contract_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("RATE_PROVIDER"),
            None,
        )
        .unwrap_err();
    assert_eq!("You need to provide init params", resp.to_string());

    let msg = RateProviderInstantiateMsg {
        asset_infos: asset_infos.clone(),
        exchange_rate: Decimal::from_ratio(1u128, 5u128),
    };

    let rate_provider = router
        .instantiate_contract(
            rate_provider_contract_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("PAIR"),
            None,
        )
        .unwrap();

    let msg = InstantiateMsg {
        asset_infos: asset_infos.clone(),
        token_code_id: token_contract_code_id,
        factory_addr: String::from("factory"),
        init_params: Some(
            to_binary(&MetaStablePoolParams {
                amp: 100,
                er_provider_addr: rate_provider.into_string(),
                er_cache_btl: 100u64,
            })
            .unwrap(),
        ),
    };

    let pair = router
        .instantiate_contract(
            pair_contract_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("PAIR"),
            None,
        )
        .unwrap();

    let res: PairInfo = router
        .wrap()
        .query_wasm_smart(pair.clone(), &QueryMsg::Pair {})
        .unwrap();
    assert_eq!("contract #1", res.contract_addr);
    assert_eq!("contract #2", res.liquidity_token);

    pair
}

#[test]
fn test_provide_and_withdraw_liquidity() {
    let owner = Addr::unchecked("owner");
    let alice_address = Addr::unchecked("alice");
    let mut router = mock_app();

    // Set Alice's balances
    router
        .init_bank_balance(
            &alice_address,
            vec![
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::new(1166u128),
                },
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::new(200u128),
                },
            ],
        )
        .unwrap();

    // Init pair
    let pair_instance = instantiate_pair(&mut router, &owner);

    let res: Result<PairInfo, _> = router.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: pair_instance.to_string(),
        msg: to_binary(&QueryMsg::Pair {}).unwrap(),
    }));
    let res = res.unwrap();

    assert_eq!(
        res.asset_infos,
        [
            AssetInfo::NativeToken {
                denom: "uusd".to_string(),
            },
            AssetInfo::NativeToken {
                denom: "uluna".to_string(),
            },
        ],
    );

    // When dealing with native tokens, the transfer should happen before the contract call, which cw-multitest doesn't support
    router
        .init_bank_balance(
            &pair_instance,
            vec![
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::new(500u128),
                },
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::new(100u128),
                },
            ],
        )
        .unwrap();

    // Provide liquidity
    let (msg, coins) = provide_liquidity_msg(Uint128::new(500), Uint128::new(100), None);
    let res = router
        .execute_contract(alice_address.clone(), pair_instance.clone(), &msg, &coins)
        .unwrap();

    assert_eq!(
        res.events[1].attributes[1],
        attr("action", "provide_liquidity")
    );
    assert_eq!(res.events[1].attributes[3], attr("receiver", "alice"),);
    assert_eq!(
        res.events[1].attributes[4],
        attr("assets", "500uusd, 100uluna")
    );
    assert_eq!(
        res.events[1].attributes[5],
        attr("share", 223u128.to_string())
    );
    assert_eq!(res.events[3].attributes[1], attr("action", "mint"));
    assert_eq!(res.events[3].attributes[2], attr("to", "alice"));
    assert_eq!(
        res.events[3].attributes[3],
        attr("amount", 223u128.to_string())
    );

    // Provide liquidity for a custom receiver
    let (msg, coins) = provide_liquidity_msg(
        Uint128::new(500),
        Uint128::new(100),
        Some("bob".to_string()),
    );
    let res = router
        .execute_contract(alice_address.clone(), pair_instance.clone(), &msg, &coins)
        .unwrap();

    assert_eq!(
        res.events[1].attributes[1],
        attr("action", "provide_liquidity")
    );
    assert_eq!(res.events[1].attributes[3], attr("receiver", "bob"),);
    assert_eq!(
        res.events[1].attributes[4],
        attr("assets", "500uusd, 100uluna")
    );
    assert_eq!(
        res.events[1].attributes[5],
        attr("share", 111u128.to_string())
    );
    assert_eq!(res.events[3].attributes[1], attr("action", "mint"));
    assert_eq!(res.events[3].attributes[2], attr("to", "bob"));
    assert_eq!(
        res.events[3].attributes[3],
        attr("amount", 111u128.to_string())
    );
}

fn provide_liquidity_msg(
    uusd_amount: Uint128,
    uluna_amount: Uint128,
    receiver: Option<String>,
) -> (ExecuteMsg, [Coin; 2]) {
    let msg = ExecuteMsg::ProvideLiquidity {
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "uusd".to_string(),
                },
                amount: uusd_amount.clone(),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "uluna".to_string(),
                },
                amount: uluna_amount.clone(),
            },
        ],
        slippage_tolerance: None,
        auto_stake: None,
        receiver,
    };

    let coins = [
        Coin {
            denom: "uluna".to_string(),
            amount: uluna_amount.clone(),
        },
        Coin {
            denom: "uusd".to_string(),
            amount: uusd_amount.clone(),
        },
    ];

    (msg, coins)
}

#[test]
fn test_compatibility_of_tokens_with_different_precision() {
    let mut app = mock_app();

    let owner = Addr::unchecked(OWNER);

    let token_code_id = store_token_code(&mut app);

    let x_amount = Uint128::new(5000000_00000);
    let y_amount = Uint128::new(1000000_0000000);
    let x_offer = Uint128::new(5_00000);
    let y_expected_return = Uint128::new(1_0000000);

    let token_name = "Xtoken";

    let init_msg = TokenInstantiateMsg {
        name: token_name.to_string(),
        symbol: token_name.to_string(),
        decimals: 5,
        initial_balances: vec![Cw20Coin {
            address: OWNER.to_string(),
            amount: x_amount + x_offer,
        }],
        mint: Some(MinterResponse {
            minter: String::from(OWNER),
            cap: None,
        }),
    };

    let token_x_instance = app
        .instantiate_contract(
            token_code_id,
            owner.clone(),
            &init_msg,
            &[],
            token_name,
            None,
        )
        .unwrap();

    let token_name = "Ytoken";

    let init_msg = TokenInstantiateMsg {
        name: token_name.to_string(),
        symbol: token_name.to_string(),
        decimals: 7,
        initial_balances: vec![Cw20Coin {
            address: OWNER.to_string(),
            amount: y_amount,
        }],
        mint: Some(MinterResponse {
            minter: String::from(OWNER),
            cap: None,
        }),
    };

    let token_y_instance = app
        .instantiate_contract(
            token_code_id,
            owner.clone(),
            &init_msg,
            &[],
            token_name,
            None,
        )
        .unwrap();

    let pair_code_id = store_pair_code(&mut app);
    let factory_code_id = store_factory_code(&mut app);
    let rate_provider_contract_code_id = store_rate_provider_code(&mut app);
    let asset_infos = [
        AssetInfo::Token {
            contract_addr: token_x_instance.clone(),
        },
        AssetInfo::Token {
            contract_addr: token_y_instance.clone(),
        },
    ];

    let init_msg = FactoryInstantiateMsg {
        fee_address: None,
        pair_configs: vec![PairConfig {
            code_id: pair_code_id,
            maker_fee_bps: 0,
            total_fee_bps: 0,
            pair_type: PairType::MetaStable {},
            is_disabled: false,
            is_generator_disabled: false,
        }],
        token_code_id,
        generator_address: Some(String::from("generator")),
        owner: String::from("owner0000"),
        whitelist_code_id: 234u64,
    };

    let factory_instance = app
        .instantiate_contract(
            factory_code_id,
            owner.clone(),
            &init_msg,
            &[],
            "FACTORY",
            None,
        )
        .unwrap();

    let msg = RateProviderInstantiateMsg {
        asset_infos: asset_infos.clone(),
        exchange_rate: Decimal::from_ratio(1u128, 5u128),
    };

    let rate_provider = app
        .instantiate_contract(
            rate_provider_contract_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("PAIR"),
            None,
        )
        .unwrap();

    let msg = FactoryExecuteMsg::CreatePair {
        pair_type: PairType::MetaStable {},
        asset_infos: asset_infos.clone(),
        init_params: Some(
            to_binary(&MetaStablePoolParams {
                amp: 100,
                er_provider_addr: rate_provider.to_string(),
                er_cache_btl: 100u64,
            })
            .unwrap(),
        ),
    };

    app.execute_contract(owner.clone(), factory_instance.clone(), &msg, &[])
        .unwrap();

    let msg = FactoryQueryMsg::Pair {
        asset_infos: asset_infos.clone(),
    };

    let res: PairInfo = app
        .wrap()
        .query_wasm_smart(&factory_instance, &msg)
        .unwrap();

    let pair_instance = res.contract_addr;

    let msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: pair_instance.to_string(),
        expires: None,
        amount: x_amount + x_offer,
    };

    app.execute_contract(owner.clone(), token_x_instance.clone(), &msg, &[])
        .unwrap();

    let msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: pair_instance.to_string(),
        expires: None,
        amount: y_amount,
    };

    app.execute_contract(owner.clone(), token_y_instance.clone(), &msg, &[])
        .unwrap();

    let msg = ExecuteMsg::ProvideLiquidity {
        assets: [
            Asset {
                info: AssetInfo::Token {
                    contract_addr: token_x_instance.clone(),
                },
                amount: x_amount,
            },
            Asset {
                info: AssetInfo::Token {
                    contract_addr: token_y_instance.clone(),
                },
                amount: y_amount,
            },
        ],
        slippage_tolerance: None,
        auto_stake: None,
        receiver: None,
    };

    app.execute_contract(owner.clone(), pair_instance.clone(), &msg, &[])
        .unwrap();

    let user = Addr::unchecked("user");

    let msg = Cw20ExecuteMsg::Send {
        contract: pair_instance.to_string(),
        msg: to_binary(&Cw20HookMsg::Swap {
            belief_price: None,
            max_spread: None,
            to: Some(user.to_string()),
        })
        .unwrap(),
        amount: x_offer,
    };

    app.execute_contract(owner.clone(), token_x_instance.clone(), &msg, &[])
        .unwrap();

    let msg = Cw20QueryMsg::Balance {
        address: user.to_string(),
    };

    let res: BalanceResponse = app
        .wrap()
        .query_wasm_smart(&token_y_instance, &msg)
        .unwrap();

    assert_eq!(res.balance, y_expected_return);
}

#[test]
fn test_if_twap_is_calculated_correctly_when_pool_idles() {
    let mut app = mock_app();

    let user1 = Addr::unchecked("user1");

    app.init_bank_balance(
        &user1,
        vec![
            Coin {
                denom: "uusd".to_string(),
                amount: Uint128::new(4666666_000000),
            },
            Coin {
                denom: "uluna".to_string(),
                amount: Uint128::new(2000000_000000),
            },
        ],
    )
    .unwrap();

    // Instantiate pair
    let pair_instance = instantiate_pair(&mut app, &user1);

    // Provide liquidity, accumulators are empty
    let (msg, coins) = provide_liquidity_msg(
        Uint128::new(1000000_000000),
        Uint128::new(1000000_000000),
        None,
    );
    app.execute_contract(user1.clone(), pair_instance.clone(), &msg, &coins)
        .unwrap();

    const BLOCKS_PER_DAY: u64 = 17280;
    const ELAPSED_SECONDS: u64 = BLOCKS_PER_DAY * 5;

    // A day later
    app.update_block(|b| {
        b.height += BLOCKS_PER_DAY;
        b.time = b.time.plus_seconds(ELAPSED_SECONDS);
    });

    // Provide liquidity, accumulators firstly filled with the same prices
    let (msg, coins) = provide_liquidity_msg(
        Uint128::new(3000000_000000),
        Uint128::new(1000000_000000),
        None,
    );
    app.execute_contract(user1.clone(), pair_instance.clone(), &msg, &coins)
        .unwrap();

    // Get current TWAP accumulator values
    let msg = QueryMsg::CumulativePrices {};
    let cpr_old: CumulativePricesResponse =
        app.wrap().query_wasm_smart(&pair_instance, &msg).unwrap();

    // A day later
    app.update_block(|b| {
        b.height += BLOCKS_PER_DAY;
        b.time = b.time.plus_seconds(ELAPSED_SECONDS);
    });

    // Get current twap accumulator values
    let msg = QueryMsg::CumulativePrices {};
    let cpr_new: CumulativePricesResponse =
        app.wrap().query_wasm_smart(&pair_instance, &msg).unwrap();

    let twap0 = cpr_new.price0_cumulative_last - cpr_old.price0_cumulative_last;
    let twap1 = cpr_new.price1_cumulative_last - cpr_old.price1_cumulative_last;

    // Prices weren't changed for the last day, uusd amount in pool = 4000000_000000, uluna = 2000000_000000
    let price_precision = Uint128::from(10u128.pow(TWAP_PRECISION.into()));
    assert_eq!(twap0 / price_precision, Uint128::new(85684)); // 1.008356286 * ELAPSED_SECONDS (86400)
    assert_eq!(twap1 / price_precision, Uint128::new(87121)); //   0.991712963 * ELAPSED_SECONDS
}

#[test]
fn create_pair_with_same_assets() {
    let mut router = mock_app();
    let owner = Addr::unchecked("owner");

    let token_contract_code_id = store_token_code(&mut router);
    let pair_contract_code_id = store_pair_code(&mut router);
    let rate_provider_contract_code_id = store_rate_provider_code(&mut router);
    let doubling_asset_infos = [
        AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
        AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
    ];

    let msg = RateProviderInstantiateMsg {
        asset_infos: doubling_asset_infos.clone(),
        exchange_rate: Decimal::from_ratio(1u128, 5u128),
    };

    let rate_provider = router
        .instantiate_contract(
            rate_provider_contract_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("PAIR"),
            None,
        )
        .unwrap_err();

    assert_eq!(rate_provider.to_string(), "Doubling assets in asset infos");

    // reinit rate provider with different assets
    let msg = RateProviderInstantiateMsg {
        asset_infos: [
            AssetInfo::NativeToken {
                denom: "uusd".to_string(),
            },
            AssetInfo::NativeToken {
                denom: "uluna".to_string(),
            },
        ],
        exchange_rate: Decimal::from_ratio(1u128, 5u128),
    };

    let rate_provider = router
        .instantiate_contract(
            rate_provider_contract_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("PAIR"),
            None,
        )
        .unwrap();

    let msg = InstantiateMsg {
        asset_infos: doubling_asset_infos.clone(),
        token_code_id: token_contract_code_id,
        factory_addr: String::from("factory"),
        init_params: Some(
            to_binary(&MetaStablePoolParams {
                amp: 100,
                er_provider_addr: rate_provider.into_string(),
                er_cache_btl: 100u64,
            })
            .unwrap(),
        ),
    };

    let resp = router
        .instantiate_contract(
            pair_contract_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("PAIR"),
            None,
        )
        .unwrap_err();

    assert_eq!(resp.to_string(), "Doubling assets in asset infos")
}

#[test]
fn update_pair_config() {
    let mut router = mock_app();
    let owner = Addr::unchecked("owner");

    let token_contract_code_id = store_token_code(&mut router);
    let pair_contract_code_id = store_pair_code(&mut router);
    let rate_provider_contract_code_id = store_rate_provider_code(&mut router);
    let asset_infos = [
        AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
        AssetInfo::NativeToken {
            denom: "uluna".to_string(),
        },
    ];

    let factory_code_id = store_factory_code(&mut router);

    let init_msg = FactoryInstantiateMsg {
        fee_address: None,
        pair_configs: vec![],
        token_code_id: token_contract_code_id,
        generator_address: Some(String::from("generator")),
        owner: owner.to_string(),
        whitelist_code_id: 234u64,
    };

    let factory_instance = router
        .instantiate_contract(
            factory_code_id,
            owner.clone(),
            &init_msg,
            &[],
            "FACTORY",
            None,
        )
        .unwrap();

    let msg = RateProviderInstantiateMsg {
        asset_infos: asset_infos.clone(),
        exchange_rate: Decimal::from_ratio(1u128, 5u128),
    };

    let rate_provider = router
        .instantiate_contract(
            rate_provider_contract_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("PAIR"),
            None,
        )
        .unwrap();

    let msg = InstantiateMsg {
        asset_infos: asset_infos.clone(),
        token_code_id: token_contract_code_id,
        factory_addr: factory_instance.to_string(),
        init_params: Some(
            to_binary(&MetaStablePoolParams {
                amp: 100,
                er_provider_addr: rate_provider.clone().into_string(),
                er_cache_btl: 100u64,
            })
            .unwrap(),
        ),
    };

    let pair = router
        .instantiate_contract(
            pair_contract_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("PAIR"),
            None,
        )
        .unwrap();

    let res: ConfigResponse = router
        .wrap()
        .query_wasm_smart(pair.clone(), &QueryMsg::Config {})
        .unwrap();

    let params: MetaStablePoolConfig = from_binary(&res.params.unwrap()).unwrap();

    assert_eq!(params.amp, Decimal::from_ratio(100u32, 1u32));
    assert_eq!(params.er_cache_btl, 100u64);
    assert_eq!(params.er_provider_addr, rate_provider.clone().into_string());

    let msg = RateProviderQueryMsg::GetExchangeRate {
        offer_asset: asset_infos[0].clone(),
        ask_asset: asset_infos[1].clone(),
    };

    let res: GetExchangeRateResponse = router
        .wrap()
        .query_wasm_smart(rate_provider.clone(), &msg)
        .unwrap();

    assert_eq!(res.exchange_rate, Decimal::from_ratio(1u128, 5u128));

    // Start changing amp with incorrect next amp
    let msg = ExecuteMsg::UpdateConfig {
        params: Some(
            to_binary(&MetaStablePoolUpdateAmp::StartChangingAmp {
                next_amp: MAX_AMP + 1,
                next_amp_time: router.block_info().time.seconds(),
            })
            .unwrap(),
        ),
        er_cache_btl: None,
        er_provider_addr: None,
    };

    let resp = router
        .execute_contract(owner.clone(), pair.clone(), &msg, &[])
        .unwrap_err();

    assert_eq!(
        resp.to_string(),
        format!(
            "Amp coefficient must be greater than 0 and less than or equal to {}",
            MAX_AMP
        )
    );

    // Start changing amp with big difference between the old and new amp value
    let msg = ExecuteMsg::UpdateConfig {
        params: Some(
            to_binary(&MetaStablePoolUpdateAmp::StartChangingAmp {
                next_amp: 100 * MAX_AMP_CHANGE + 1,
                next_amp_time: router.block_info().time.seconds(),
            })
            .unwrap(),
        ),
        er_cache_btl: None,
        er_provider_addr: None,
    };

    let resp = router
        .execute_contract(owner.clone(), pair.clone(), &msg, &[])
        .unwrap_err();

    assert_eq!(
        resp.to_string(),
        format!(
            "The difference between the old and new amp value must not exceed {} times",
            MAX_AMP_CHANGE
        )
    );

    // Start changing amp before the MIN_AMP_CHANGING_TIME has elapsed
    let msg = ExecuteMsg::UpdateConfig {
        params: Some(
            to_binary(&MetaStablePoolUpdateAmp::StartChangingAmp {
                next_amp: 250,
                next_amp_time: router.block_info().time.seconds(),
            })
            .unwrap(),
        ),
        er_cache_btl: None,
        er_provider_addr: None,
    };

    let resp = router
        .execute_contract(owner.clone(), pair.clone(), &msg, &[])
        .unwrap_err();

    assert_eq!(
        resp.to_string(),
        format!(
            "Amp coefficient cannot be changed more often than once per {} seconds",
            MIN_AMP_CHANGING_TIME
        )
    );

    // Start increasing amp
    router.update_block(|b| {
        b.time = b.time.plus_seconds(MIN_AMP_CHANGING_TIME);
    });

    let msg = ExecuteMsg::UpdateConfig {
        params: Some(
            to_binary(&MetaStablePoolUpdateAmp::StartChangingAmp {
                next_amp: 250,
                next_amp_time: router.block_info().time.seconds() + MIN_AMP_CHANGING_TIME,
            })
            .unwrap(),
        ),
        er_cache_btl: None,
        er_provider_addr: None,
    };

    router
        .execute_contract(owner.clone(), pair.clone(), &msg, &[])
        .unwrap();

    router.update_block(|b| {
        b.time = b.time.plus_seconds(MIN_AMP_CHANGING_TIME / 2);
    });

    let res: ConfigResponse = router
        .wrap()
        .query_wasm_smart(pair.clone(), &QueryMsg::Config {})
        .unwrap();

    let params: MetaStablePoolConfig = from_binary(&res.params.unwrap()).unwrap();

    assert_eq!(params.amp, Decimal::from_ratio(175u32, 1u32));

    router.update_block(|b| {
        b.time = b.time.plus_seconds(MIN_AMP_CHANGING_TIME / 2);
    });

    let res: ConfigResponse = router
        .wrap()
        .query_wasm_smart(pair.clone(), &QueryMsg::Config {})
        .unwrap();

    let params: MetaStablePoolConfig = from_binary(&res.params.unwrap()).unwrap();

    assert_eq!(params.amp, Decimal::from_ratio(250u32, 1u32));

    // Start decreasing amp
    router.update_block(|b| {
        b.time = b.time.plus_seconds(MIN_AMP_CHANGING_TIME);
    });

    let msg = ExecuteMsg::UpdateConfig {
        params: Some(
            to_binary(&MetaStablePoolUpdateAmp::StartChangingAmp {
                next_amp: 50,
                next_amp_time: router.block_info().time.seconds() + MIN_AMP_CHANGING_TIME,
            })
            .unwrap(),
        ),
        er_cache_btl: None,
        er_provider_addr: None,
    };

    router
        .execute_contract(owner.clone(), pair.clone(), &msg, &[])
        .unwrap();

    router.update_block(|b| {
        b.time = b.time.plus_seconds(MIN_AMP_CHANGING_TIME / 2);
    });

    let res: ConfigResponse = router
        .wrap()
        .query_wasm_smart(pair.clone(), &QueryMsg::Config {})
        .unwrap();

    let params: MetaStablePoolConfig = from_binary(&res.params.unwrap()).unwrap();

    assert_eq!(params.amp, Decimal::from_ratio(150u32, 1u32));

    // Stop changing amp
    let msg = ExecuteMsg::UpdateConfig {
        params: Some(to_binary(&MetaStablePoolUpdateAmp::StopChangingAmp {}).unwrap()),
        er_cache_btl: None,
        er_provider_addr: None,
    };

    router
        .execute_contract(owner.clone(), pair.clone(), &msg, &[])
        .unwrap();

    router.update_block(|b| {
        b.time = b.time.plus_seconds(MIN_AMP_CHANGING_TIME / 2);
    });

    let res: ConfigResponse = router
        .wrap()
        .query_wasm_smart(pair.clone(), &QueryMsg::Config {})
        .unwrap();

    let params: MetaStablePoolConfig = from_binary(&res.params.unwrap()).unwrap();

    assert_eq!(params.amp, Decimal::from_ratio(150u32, 1u32));

    // change rate provider
    let rate_provider_contract_code_id = store_rate_provider_code(&mut router);

    let msg = RateProviderInstantiateMsg {
        asset_infos: asset_infos.clone(),
        exchange_rate: Decimal::from_ratio(1u128, 10u128),
    };

    let new_rate_provider = router
        .instantiate_contract(
            rate_provider_contract_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("PAIR"),
            None,
        )
        .unwrap();

    let msg = ExecuteMsg::UpdateConfig {
        params: None,
        er_cache_btl: Some(555u64),
        er_provider_addr: Some(new_rate_provider.clone().into_string()),
    };

    router
        .execute_contract(owner.clone(), pair.clone(), &msg, &[])
        .unwrap();

    let res: ConfigResponse = router
        .wrap()
        .query_wasm_smart(pair.clone(), &QueryMsg::Config {})
        .unwrap();

    let params: MetaStablePoolConfig = from_binary(&res.params.unwrap()).unwrap();

    assert_eq!(params.er_cache_btl, 555u64);
    assert_eq!(
        params.er_provider_addr,
        new_rate_provider.clone().into_string()
    );

    let msg = RateProviderQueryMsg::GetExchangeRate {
        offer_asset: asset_infos[0].clone(),
        ask_asset: asset_infos[1].clone(),
    };

    let res: GetExchangeRateResponse = router
        .wrap()
        .query_wasm_smart(new_rate_provider, &msg)
        .unwrap();

    assert_eq!(res.exchange_rate, Decimal::from_ratio(1u128, 10u128));
}
