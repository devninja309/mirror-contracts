use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{to_binary, Decimal, HumanAddr, StdError, Uint128, WasmQuery};

use crate::contract::{handle, init, query_collateral_info, query_collateral_price, query_config};
use crate::mock_querier::mock_dependencies;
use mirror_protocol::collateral_oracle::{
    CollateralInfoResponse, CollateralPriceResponse, HandleMsg, InitMsg,
};
use mirror_protocol::oracle::QueryMsg as OracleQueryMsg;
use terraswap::asset::AssetInfo;
use terraswap::pair::QueryMsg as TerraswapPairQueryMsg;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(20, &[]);

    let msg = InitMsg {
        owner: HumanAddr("owner0000".to_string()),
        mint_contract: HumanAddr("mint0000".to_string()),
        factory_contract: HumanAddr("factory0000".to_string()),
        base_denom: "uusd".to_string(),
    };

    let env = mock_env("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let res = init(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let value = query_config(&deps).unwrap();
    assert_eq!("owner0000", value.owner.as_str());
    assert_eq!("mint0000", value.mint_contract.as_str());
    assert_eq!("factory0000", value.factory_contract.as_str());
    assert_eq!("uusd", value.base_denom.as_str());
}

#[test]
fn update_config() {
    let mut deps = mock_dependencies(20, &[]);

    let msg = InitMsg {
        owner: HumanAddr("owner0000".to_string()),
        mint_contract: HumanAddr("mint0000".to_string()),
        factory_contract: HumanAddr("factory0000".to_string()),
        base_denom: "uusd".to_string(),
    };

    let env = mock_env("addr0000", &[]);
    let _res = init(&mut deps, env, msg).unwrap();

    // update owner
    let env = mock_env("owner0000", &[]);
    let msg = HandleMsg::UpdateConfig {
        owner: Some(HumanAddr("owner0001".to_string())),
        mint_contract: Some(HumanAddr("mint0001".to_string())),
        factory_contract: Some(HumanAddr("factory0001".to_string())),
        base_denom: Some("uluna".to_string()),
    };

    let res = handle(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let value = query_config(&deps).unwrap();
    assert_eq!("owner0001", value.owner.as_str());
    assert_eq!("mint0001", value.mint_contract.as_str());
    assert_eq!("factory0001", value.factory_contract.as_str());
    assert_eq!("uluna", value.base_denom.as_str());

    // Unauthorized err
    let env = mock_env("owner0000", &[]);
    let msg = HandleMsg::UpdateConfig {
        owner: None,
        mint_contract: None,
        factory_contract: None,
        base_denom: None,
    };

    let res = handle(&mut deps, env, msg);
    match res {
        Err(StdError::Unauthorized { .. }) => {}
        _ => panic!("Must return unauthorized error"),
    }
}

#[test]
fn register_collateral() {
    let mut deps = mock_dependencies(20, &[]);
    deps.querier.with_oracle_price(&[
        (&"uusd".to_string(), &Decimal::one()),
        (&"mTSLA".to_string(), &Decimal::percent(100)),
    ]);

    let msg = InitMsg {
        owner: HumanAddr("owner0000".to_string()),
        mint_contract: HumanAddr("mint0000".to_string()),
        factory_contract: HumanAddr("factory0000".to_string()),
        base_denom: "uusd".to_string(),
    };

    let env = mock_env("addr0000", &[]);
    let _res = init(&mut deps, env, msg).unwrap();

    let wasm_query: WasmQuery = WasmQuery::Smart {
        contract_addr: HumanAddr::from("oracle0000"),
        msg: to_binary(&OracleQueryMsg::Price {
            base_asset: "uusd".to_string(),
            quote_asset: "mTSLA".to_string(),
        })
        .unwrap(),
    };
    let query_request = to_binary(&wasm_query).unwrap();

    let msg = HandleMsg::RegisterCollateralAsset {
        asset: AssetInfo::Token {
            contract_addr: HumanAddr::from("mTSLA"),
        },
        collateral_premium: Decimal::percent(50),
        query_request: query_request.clone(),
    };

    // unauthorized attempt
    let env = mock_env("addr0000", &[]);
    let res = handle(&mut deps, env, msg.clone()).unwrap_err();
    assert_eq!(res, StdError::unauthorized());

    // successfull attempt
    let env = mock_env("owner0000", &[]);
    let res = handle(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // query collateral info
    let query_res = query_collateral_info(&deps, "mTSLA".to_string()).unwrap();
    assert_eq!(
        query_res,
        CollateralInfoResponse {
            asset: "mTSLA".to_string(),
            query_request: wasm_query,
            collateral_premium: Decimal::percent(50),
            is_revoked: false,
        }
    )
}

#[test]
fn update_collateral() {
    let mut deps = mock_dependencies(20, &[]);
    deps.querier.with_oracle_price(&[
        (&"uusd".to_string(), &Decimal::one()),
        (&"mTSLA".to_string(), &Decimal::percent(100)),
    ]);

    let msg = InitMsg {
        owner: HumanAddr("owner0000".to_string()),
        mint_contract: HumanAddr("mint0000".to_string()),
        factory_contract: HumanAddr("factory0000".to_string()),
        base_denom: "uusd".to_string(),
    };

    let env = mock_env("addr0000", &[]);
    let _res = init(&mut deps, env, msg).unwrap();

    let wasm_query: WasmQuery = WasmQuery::Smart {
        contract_addr: HumanAddr::from("oracle0000"),
        msg: to_binary(&OracleQueryMsg::Price {
            base_asset: "uusd".to_string(),
            quote_asset: "mTSLA".to_string(),
        })
        .unwrap(),
    };
    let query_request = to_binary(&wasm_query).unwrap();

    let msg = HandleMsg::RegisterCollateralAsset {
        asset: AssetInfo::Token {
            contract_addr: HumanAddr::from("mTSLA"),
        },
        collateral_premium: Decimal::percent(50),
        query_request: query_request.clone(),
    };

    // successfull attempt
    let env = mock_env("owner0000", &[]);
    let res = handle(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // query collateral info
    let query_res = query_collateral_info(&deps, "mTSLA".to_string()).unwrap();
    assert_eq!(
        query_res,
        CollateralInfoResponse {
            asset: "mTSLA".to_string(),
            query_request: wasm_query,
            collateral_premium: Decimal::percent(50),
            is_revoked: false,
        }
    );

    let new_wasm_query: WasmQuery = WasmQuery::Smart {
        contract_addr: HumanAddr::from("oracle0001"), // change contract_addr
        msg: to_binary(&OracleQueryMsg::Price {
            base_asset: "uusd".to_string(),
            quote_asset: "mTSLA".to_string(),
        })
        .unwrap(),
    };
    let new_query_request = to_binary(&new_wasm_query).unwrap();

    // update collateral query
    let msg = HandleMsg::UpdateCollateralQuery {
        asset: AssetInfo::Token {
            contract_addr: HumanAddr::from("mTSLA"),
        },
        query_request: new_query_request.clone(),
    };

    // unauthorized attempt
    let env = mock_env("factory0000", &[]);
    let res = handle(&mut deps, env, msg.clone()).unwrap_err();
    assert_eq!(res, StdError::unauthorized());

    // successfull attempt
    let env = mock_env("owner0000", &[]);
    let res = handle(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // query the updated collateral
    let query_res = query_collateral_info(&deps, "mTSLA".to_string()).unwrap();
    assert_eq!(
        query_res,
        CollateralInfoResponse {
            asset: "mTSLA".to_string(),
            query_request: new_wasm_query.clone(),
            collateral_premium: Decimal::percent(50),
            is_revoked: false,
        }
    );

    // update collateral premium
    let msg = HandleMsg::UpdateCollateralPremium {
        asset: AssetInfo::Token {
            contract_addr: HumanAddr::from("mTSLA"),
        },
        collateral_premium: Decimal::percent(60),
    };

    // unauthorized attempt
    let env = mock_env("owner0000", &[]);
    let res = handle(&mut deps, env, msg.clone()).unwrap_err();
    assert_eq!(res, StdError::unauthorized());

    // successfull attempt
    let env = mock_env("factory0000", &[]);
    let res = handle(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // query the updated collateral
    let query_res = query_collateral_info(&deps, "mTSLA".to_string()).unwrap();
    assert_eq!(
        query_res,
        CollateralInfoResponse {
            asset: "mTSLA".to_string(),
            query_request: new_wasm_query,
            collateral_premium: Decimal::percent(60),
            is_revoked: false,
        }
    )
}

#[test]
fn get_oracle_price() {
    let mut deps = mock_dependencies(20, &[]);
    deps.querier.with_oracle_price(&[
        (&"uusd".to_string(), &Decimal::one()),
        (&"mTSLA".to_string(), &Decimal::percent(100)),
    ]);

    let msg = InitMsg {
        owner: HumanAddr("owner0000".to_string()),
        mint_contract: HumanAddr("mint0000".to_string()),
        factory_contract: HumanAddr("factory0000".to_string()),
        base_denom: "uusd".to_string(),
    };

    let env = mock_env("addr0000", &[]);
    let _res = init(&mut deps, env, msg).unwrap();

    let msg = HandleMsg::RegisterCollateralAsset {
        asset: AssetInfo::Token {
            contract_addr: HumanAddr::from("mTSLA"),
        },
        collateral_premium: Decimal::percent(50),
        query_request: to_binary(&WasmQuery::Smart {
            contract_addr: HumanAddr::from("oracle0000"),
            msg: to_binary(&OracleQueryMsg::Price {
                base_asset: "uusd".to_string(),
                quote_asset: "mTSLA".to_string(),
            })
            .unwrap(),
        })
        .unwrap(),
    };

    let env = mock_env("owner0000", &[]);
    let _res = handle(&mut deps, env, msg).unwrap();

    // attempt to query price
    let query_res = query_collateral_price(&deps, "mTSLA".to_string()).unwrap();
    assert_eq!(
        query_res,
        CollateralPriceResponse {
            asset: "mTSLA".to_string(),
            rate: Decimal::percent(100),
            collateral_premium: Decimal::percent(50),
            is_revoked: false,
        }
    );
}

#[test]
fn get_terraswap_price() {
    let mut deps = mock_dependencies(20, &[]);
    deps.querier.with_terraswap_pools(&[(
        &HumanAddr::from("ustancpair0000"),
        (&Uint128(1u128), &Uint128(100u128)),
    )]);

    let msg = InitMsg {
        owner: HumanAddr("owner0000".to_string()),
        mint_contract: HumanAddr("mint0000".to_string()),
        factory_contract: HumanAddr("factory0000".to_string()),
        base_denom: "uusd".to_string(),
    };

    let env = mock_env("addr0000", &[]);
    let _res = init(&mut deps, env, msg).unwrap();

    let msg = HandleMsg::RegisterCollateralAsset {
        asset: AssetInfo::Token {
            contract_addr: HumanAddr::from("anc"),
        },
        collateral_premium: Decimal::percent(50),
        query_request: to_binary(&WasmQuery::Smart {
            contract_addr: HumanAddr::from("ustancpair0000"),
            msg: to_binary(&TerraswapPairQueryMsg::Pool {}).unwrap(),
        })
        .unwrap(),
    };

    let env = mock_env("owner0000", &[]);
    let _res = handle(&mut deps, env, msg).unwrap();

    // attempt to query price
    let query_res = query_collateral_price(&deps, "anc".to_string()).unwrap();
    assert_eq!(
        query_res,
        CollateralPriceResponse {
            asset: "anc".to_string(),
            rate: Decimal::from_ratio(1u128, 100u128),
            collateral_premium: Decimal::percent(50),
            is_revoked: false,
        }
    );
}

#[test]
fn revoke_collateral() {
    let mut deps = mock_dependencies(20, &[]);
    deps.querier.with_oracle_price(&[
        (&"uusd".to_string(), &Decimal::one()),
        (&"mTSLA".to_string(), &Decimal::percent(100)),
    ]);

    let msg = InitMsg {
        owner: HumanAddr("owner0000".to_string()),
        mint_contract: HumanAddr("mint0000".to_string()),
        factory_contract: HumanAddr("factory0000".to_string()),
        base_denom: "uusd".to_string(),
    };

    let env = mock_env("addr0000", &[]);
    let _res = init(&mut deps, env, msg).unwrap();

    let wasm_query: WasmQuery = WasmQuery::Smart {
        contract_addr: HumanAddr::from("oracle0000"),
        msg: to_binary(&OracleQueryMsg::Price {
            base_asset: "uusd".to_string(),
            quote_asset: "mTSLA".to_string(),
        })
        .unwrap(),
    };

    let msg = HandleMsg::RegisterCollateralAsset {
        asset: AssetInfo::Token {
            contract_addr: HumanAddr::from("mTSLA"),
        },
        collateral_premium: Decimal::percent(50),
        query_request: to_binary(&wasm_query).unwrap(),
    };

    let env = mock_env("owner0000", &[]);
    let _res = handle(&mut deps, env, msg).unwrap();

    // attempt to query price
    let query_res = query_collateral_price(&deps, "mTSLA".to_string()).unwrap();
    assert_eq!(
        query_res,
        CollateralPriceResponse {
            asset: "mTSLA".to_string(),
            rate: Decimal::percent(100),
            collateral_premium: Decimal::percent(50),
            is_revoked: false,
        }
    );

    // revoke the asset
    let msg = HandleMsg::RevokeCollateralAsset {
        asset: AssetInfo::Token {
            contract_addr: HumanAddr::from("mTSLA"),
        },
    };

    // unauthorized attempt
    let env = mock_env("factory0000", &[]);
    let res = handle(&mut deps, env, msg.clone()).unwrap_err();
    assert_eq!(res, StdError::unauthorized());

    // successfull attempt
    let env = mock_env("owner0000", &[]);
    let res = handle(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // query the revoked collateral
    let query_res = query_collateral_info(&deps, "mTSLA".to_string()).unwrap();
    assert_eq!(
        query_res,
        CollateralInfoResponse {
            asset: "mTSLA".to_string(),
            query_request: wasm_query,
            collateral_premium: Decimal::percent(50),
            is_revoked: true,
        }
    );

    // attempt to query price of revoked asset
    let query_res = query_collateral_price(&deps, "mTSLA".to_string()).unwrap();
    assert_eq!(
        query_res,
        CollateralPriceResponse {
            asset: "mTSLA".to_string(),
            rate: Decimal::percent(100),
            collateral_premium: Decimal::percent(50),
            is_revoked: true,
        }
    );
}
