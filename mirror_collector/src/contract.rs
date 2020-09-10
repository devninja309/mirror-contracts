use cosmwasm_std::{
    log, to_binary, Api, Binary, Coin, CosmosMsg, Decimal, Env, Extern, HandleResponse,
    HandleResult, InitResponse, Querier, StdResult, Storage, Uint128, WasmMsg,
};

use crate::msg::{ConfigResponse, HandleMsg, InitMsg, MarketHandleMsg, QueryMsg};
use crate::querier::{load_balance, load_token_balance, load_whitelist_info, WhitelistInfo};
use crate::state::{read_config, store_config, Config};

use cw20::Cw20HandleMsg;
use terra_cosmwasm::TerraQuerier;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    store_config(
        &mut deps.storage,
        &Config {
            factory_contract: deps.api.canonical_address(&msg.factory_contract)?,
            mirror_symbol: msg.mirror_symbol,
            collateral_denom: msg.collateral_denom,
        },
    )?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Convert { symbol } => try_convert(deps, env, symbol),
        HandleMsg::Send {} => try_send(deps, env),
    }
}

// Anyone can execute convert function to convert rewards to staking token
pub fn try_convert<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    symbol: String,
) -> HandleResult {
    let config: Config = read_config(&deps.storage)?;
    let whitelist_info: WhitelistInfo = load_whitelist_info(
        &deps,
        &deps.api.human_address(&config.factory_contract)?,
        symbol.to_string(),
    )?;

    let mut messages: Vec<CosmosMsg> = vec![];
    if config.mirror_symbol == symbol {
        // uusd => staking token
        let amount = load_balance(
            &deps,
            &env.contract.address,
            config.collateral_denom.to_string(),
        )?;

        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.human_address(&whitelist_info.market_contract)?,
            msg: to_binary(&MarketHandleMsg::Buy { max_spread: None })?,
            send: vec![deduct_tax(
                deps,
                Coin {
                    denom: config.collateral_denom,
                    amount,
                },
            )?],
        }));
    } else {
        // asset token => uusd
        let amount = load_token_balance(
            &deps,
            &deps.api.human_address(&whitelist_info.token_contract)?,
            &deps.api.canonical_address(&env.contract.address)?,
        )?;

        let market_addr = deps.api.human_address(&whitelist_info.market_contract)?;
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.human_address(&whitelist_info.token_contract)?,
            msg: to_binary(&Cw20HandleMsg::IncreaseAllowance {
                spender: market_addr.clone(),
                amount,
                expires: None,
            })?,
            send: vec![],
        }));

        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: market_addr,
            msg: to_binary(&MarketHandleMsg::Sell {
                amount,
                max_spread: None,
            })?,
            send: vec![],
        }));
    }

    Ok(HandleResponse {
        messages,
        log: vec![log("action", "convert"), log("symbol", symbol)],
        data: None,
    })
}

// Anyone can execute send function to receive staking token rewards
pub fn try_send<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> HandleResult {
    let config: Config = read_config(&deps.storage)?;
    let whitelist_info: WhitelistInfo = load_whitelist_info(
        &deps,
        &deps.api.human_address(&config.factory_contract)?,
        config.mirror_symbol,
    )?;

    let amount = load_token_balance(
        &deps,
        &deps.api.human_address(&whitelist_info.token_contract)?,
        &deps.api.canonical_address(&env.contract.address)?,
    )?;

    Ok(HandleResponse {
        messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.human_address(&whitelist_info.token_contract)?,
            msg: to_binary(&Cw20HandleMsg::Transfer {
                recipient: deps.api.human_address(&whitelist_info.staking_contract)?,
                amount,
            })?,
            send: vec![],
        })],
        log: vec![log("action", "send"), log("amount", amount.to_string())],
        data: None,
    })
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

pub fn query_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<ConfigResponse> {
    let state = read_config(&deps.storage)?;
    let resp = ConfigResponse {
        factory_contract: deps.api.human_address(&state.factory_contract)?,
        mirror_symbol: state.mirror_symbol,
        collateral_denom: state.collateral_denom,
    };

    Ok(resp)
}

pub fn deduct_tax<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    coin: Coin,
) -> StdResult<Coin> {
    let terra_querier = TerraQuerier::new(&deps.querier);
    let tax_rate: Decimal = terra_querier.query_tax_rate()?;
    let tax_cap: Uint128 = terra_querier.query_tax_cap(coin.denom.to_string())?;
    Ok(Coin {
        amount: std::cmp::max(
            (coin.amount - coin.amount * tax_rate)?,
            (coin.amount - tax_cap).unwrap_or_else(|_| Uint128::zero()),
        ),
        ..coin
    })
}
