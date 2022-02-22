use astroport::asset::{Asset, AssetInfo, PairInfo};
use astroport::factory::{
    ExecuteMsg as FactoryExecuteMsg, InstantiateMsg as FactoryInstantiateMsg, PairConfig, PairType,
    QueryMsg as FactoryQueryMsg,
};
use astroport::router::SwapOperation;
use astroport::token::InstantiateMsg as TokenInstantiateMsg;
use astroport::{factory, pair, router, token};
use astroport_tests::base::{BaseAstroportTestInitMessage, BaseAstroportTestPackage};
use cosmwasm_std::testing::{mock_env, MockApi, MockStorage};
use cosmwasm_std::{
    attr, to_binary, Addr, Coin, Decimal, QueryRequest, StdResult, Uint128, WasmQuery,
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

fn init_astroport_test_package(router: &mut TerraApp) -> StdResult<BaseAstroportTestPackage> {
    let base_msg = BaseAstroportTestInitMessage {
        owner: Addr::unchecked(OWNER),
    };

    Ok(BaseAstroportTestPackage::init_all(router, base_msg))
}

#[test]
fn test_swap() {
    let mut router_app = mock_app();
    let router_app_ref = &mut router_app;
    let owner = Addr::unchecked(OWNER);

    let factory_contract = Box::new(
        ContractWrapper::new_with_empty(
            astroport_factory::contract::execute,
            astroport_factory::contract::instantiate,
            astroport_factory::contract::query,
        )
        .with_reply_empty(astroport_factory::contract::reply),
    );

    let factory_code_id = router_app.store_code(factory_contract);

    let pair_bluna_contract = Box::new(
        ContractWrapper::new_with_empty(
            astroport_pair_stable_bluna::contract::execute,
            astroport_pair_stable_bluna::contract::instantiate,
            astroport_pair_stable_bluna::contract::query,
        )
        .with_reply_empty(astroport_pair_stable_bluna::contract::reply),
    );
    let pair_bluna_code_id = router_app.store_code(pair_bluna_contract);

    let astro_token_contract = Box::new(ContractWrapper::new_with_empty(
        astroport_token::contract::execute,
        astroport_token::contract::instantiate,
        astroport_token::contract::query,
    ));

    let astro_token_code_id = router_app.store_code(astro_token_contract);

    let init_msg = factory::InstantiateMsg {
        fee_address: None,
        pair_configs: vec![PairConfig {
            code_id: pair_bluna_code_id,
            maker_fee_bps: 0,
            total_fee_bps: 0,
            pair_type: PairType::Stable {},
            is_disabled: false,
            is_generator_disabled: false,
        }],
        token_code_id: astro_token_code_id,
        generator_address: Some(String::from("generator")),
        owner: String::from("owner0000"),
        whitelist_code_id: 23u64,
    };

    let factory_instance = router_app
        .instantiate_contract(
            factory_code_id,
            Addr::unchecked(owner.clone()),
            &init_msg,
            &[],
            "factory",
            None,
        )
        .unwrap();

    let token_name = "bluna";

    let init_msg = token::InstantiateMsg {
        name: token_name.to_string(),
        symbol: token_name.to_string(),
        decimals: 5,
        initial_balances: vec![Cw20Coin {
            address: OWNER.to_string(),
            amount: Uint128::new(1000000000),
        }],
        mint: Some(MinterResponse {
            minter: String::from(OWNER),
            cap: None,
        }),
    };

    let bluna_token_instance = router_app
        .instantiate_contract(
            astro_token_code_id,
            owner.clone(),
            &init_msg,
            &[],
            token_name,
            None,
        )
        .unwrap();

    let msg = pair::InstantiateMsg {
        asset_infos: [
            AssetInfo::Token {
                contract_addr: bluna_token_instance,
            },
            AssetInfo::NativeToken {
                denom: "uusd".to_string(),
            },
        ],
        token_code_id: astro_token_code_id,
        factory_addr: factory_instance.to_string(),
        init_params: None,
    };

    let pair_bluna_instance = router_app
        .instantiate_contract(
            pair_bluna_code_id,
            owner.clone(),
            bluna_instance & msg,
            &[],
            String::from("PAIR"),
            None,
        )
        .unwrap();

    let router_contract = Box::new(ContractWrapper::new(
        astroport_router::contract::execute,
        astroport_router::contract::instantiate,
        astroport_router::contract::query,
    ));

    let router_code_id = router_app.store_code(router_contract);

    let msg = router::InstantiateMsg {
        astroport_factory: factory_instance.to_string(),
    };

    let router_instance = router_app
        .instantiate_contract(
            router_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("ASTRO"),
            None,
        )
        .unwrap();

    let msg = router::ExecuteMsg::ExecuteSwapOperation {
        operation: SwapOperation::AstroSwap {
            offer_asset_info: AssetInfo::Token {
                contract_addr: pair_bluna_instance,
            },
            ask_asset_info: AssetInfo::NativeToken {
                denom: "uusd".to_string(),
            },
        },
        to: None,
        max_spread: Some(Decimal::percent(5)),
    };

    router_app
        .execute_contract(router_instance.clone(), router_instance.clone(), &msg, &[])
        .unwrap();
}
