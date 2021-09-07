use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{
    coins, from_binary, to_binary, Addr, Uint128, Response, CosmosMsg,
    WasmMsg,
};
use leveraged_pools::pool::{
    InstantiateMsg, QueryMsg, HyperparametersResponse,
    PriceHistoryResponse, PoolStateResponse, ExecuteMsg,
    Cw20HookMsg, LiquidityPositionResponse, ProviderPosition,
};
use cw20::{Cw20ReceiveMsg, Cw20ExecuteMsg};
use crate::contract::{instantiate, query, execute};
use crate::testing::mock_querier::{mock_dependencies, OwnedMockDeps};

/* Create a 2x pool from a CW20
 * + TS liquidity at 100:1 mTSLA:UST
 * + Minimum protocol ratio 2.5
 * + Rebalance ratio 2.0
 * + 0.5% premium on minting 2x assets
 * + 10% premium on rebalanced positions
 */
fn mtsla_ust_2x_init(deps: &mut OwnedMockDeps) -> Response {
    /* Create a TerraSwap pool and fill it with mTSLA and uusd */
    deps.querier.with_terraswap_pools(&[
        (
            &"mTSLA-UST".to_string(),
            (
                &"uusd".to_string(),
                &Uint128::from(1_000_000_000_000u128),
                &"mTSLA".to_string(),
                &Uint128::from(1_000_000_000u128),
            ),
        ),
    ]);

    /* Hyperparameters */
    let msg = InstantiateMsg {
        leverage_amount: Uint128::new(2_000_000),
        minimum_protocol_ratio: Uint128::new(2_500_000),
        rebalance_ratio: Uint128::new(2_000_000),
        mint_premium: Uint128::new(0_500_000),
        rebalance_premium: Uint128::new(10_000_000),
        /* Previous terraswap pool */
        terraswap_pair_addr: String::from("mTSLA-UST"),
        /* Contract of the asset that is being leveraged */
        leveraged_asset_addr: String::from("mTSLA"),
    };

    /* Initialize leveraged pool */
    let info = mock_info("leveraged", &coins(1000, "big_ones"));
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap()
}

#[test]
fn proper_init() {
    let mut deps = mock_dependencies(&[]);

    let res = mtsla_ust_2x_init(&mut deps);
    assert_eq!(0, res.messages.len());

    /* Query hyperparameters and validate they are as we set them to be */
    let msg = QueryMsg::Hyperparameters { };
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    let hyper_p: HyperparametersResponse = from_binary(&res).unwrap();
    assert_eq!(hyper_p.leverage_amount, Uint128::new(2_000_000));
    assert_eq!(hyper_p.rebalance_ratio, Uint128::new(2_000_000));
    assert_eq!(hyper_p.minimum_protocol_ratio, Uint128::new(2_500_000));
    assert_eq!(hyper_p.mint_premium, Uint128::new(0_500_000));
    assert_eq!(hyper_p.rebalance_premium, Uint128::new(10_000_000));
    assert_eq!(hyper_p.terraswap_pair_addr, Addr::unchecked("mTSLA-UST"));
    assert_eq!(hyper_p.leveraged_asset_addr, Addr::unchecked("mTSLA"));

    /* Check that pool state was also initialized correctly */
    let msg = QueryMsg::PoolState { };
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    let pool_state: PoolStateResponse = from_binary(&res).unwrap();
    assert_eq!(pool_state.assets_in_reserve, Uint128::zero());

    /* Assert that inital price was correctly queried from mocked TerraSwap */
    assert_eq!(pool_state.opening_snapshot.timestamp > 0, true);
    assert_eq!(pool_state.opening_snapshot.asset_price.u128() / 1_000_000, 1_000);

    /* Query asset price history */
    let msg = QueryMsg::PriceHistory { };
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    let u_price_history: PriceHistoryResponse = from_binary(&res).unwrap();
    let genesis_snapshot = u_price_history.price_history;
    assert_eq!(genesis_snapshot.len(), 1);

    let genesis_snapshot = genesis_snapshot[0];
    /* At genesis leveraged price should equal asset price */
    assert_eq!(genesis_snapshot.asset_price, genesis_snapshot.leveraged_price);
    /* Verify genesis snapshot price is correct */
    assert_eq!(genesis_snapshot.asset_price.u128() / 1_000_000, 1_000);
}

#[test]
fn proper_lp() {
    let mut deps = mock_dependencies(&[]);

    /* mTSLA pool init */
    mtsla_ust_2x_init(&mut deps);

    /* Provide 100 mTSLA as liquidity */
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "provider".to_string(),
        amount: Uint128::new(100_000_000),
        msg: to_binary(&Cw20HookMsg::ProvideLiquidity {
        }).unwrap(),
    });
    execute(deps.as_mut(), mock_env(), mock_info(
        "mTSLA", &[]), msg).unwrap();

    /* Verify 100 mTSLA were recorded as pool liquidity */
    let res = query(deps.as_ref(), mock_env(), QueryMsg::PoolState{ }).unwrap();
    let pool_state: PoolStateResponse = from_binary(&res).unwrap();
    assert_eq!(pool_state.assets_in_reserve, Uint128::new(100_000_000));
    assert_eq!(pool_state.total_asset_pool_share, Uint128::new(100_000_000));

    /* Verify the pool recorded our position */
    let bin = &query(deps.as_ref(), mock_env(), QueryMsg::LiquidityPosition {
        address: Addr::unchecked("provider"),
    }).unwrap();
    let res: LiquidityPositionResponse = from_binary(&bin).unwrap();
    let position: ProviderPosition = res.position;

    /* We own the entire pool of course */
    assert_eq!(position.asset_pool_total_share, Uint128::new(100_000_000));
    assert_eq!(position.asset_pool_partial_share, Uint128::new(100_000_000));

    /* Someone else provides 100 mTSLA as well */
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "someone_else".to_string(),
        amount: Uint128::new(100_000_000),
        msg: to_binary(&Cw20HookMsg::ProvideLiquidity {
        }).unwrap(),
    });
    execute(deps.as_mut(), mock_env(), mock_info(
        "mTSLA", &[]), msg).unwrap();

    /* Check our LP position after someone else deposits 100 mTSLA */
    let bin = &query(deps.as_ref(), mock_env(), QueryMsg::LiquidityPosition {
        address: Addr::unchecked("provider"),
    }).unwrap();
    let res: LiquidityPositionResponse = from_binary(&bin).unwrap();
    let position: ProviderPosition = res.position;

    /* Assert that we now only own half the pool */
    assert_eq!(position.asset_pool_total_share, Uint128::new(200_000_000));
    assert_eq!(position.asset_pool_partial_share, Uint128::new(100_000_000));

    /* Attempt to withdraw our liquidity */
    let msg = ExecuteMsg::WithdrawLiquidity {
        share_of_pool: Uint128::new(100_000_000),
    };
    let res = execute(deps.as_mut(), mock_env(), mock_info(
        "provider", &[]), msg).unwrap();

    /* Extract Cw20ExecuteMsg::Transfer from response */
    let (denom, receipt) = match &res.messages[0].msg {
        CosmosMsg::Wasm(w) => match w {
                WasmMsg::Execute{ contract_addr, msg, .. } =>
                    (contract_addr, msg),
                _ => panic!("Invalid WithdrawLiquidity response"),
            },
        _ => panic!("Invalid WithdrawLiquidity response"),
    };
    let receipt_msg: Cw20ExecuteMsg = from_binary(&receipt).unwrap();
    let (recipient, amount) = match receipt_msg {
        Cw20ExecuteMsg::Transfer{ recipient, amount } => (recipient, amount),
        _ => panic!("Invalid WithdrawLiquidity response"),
    };

    /* Assert that we are credited our funds we withdrew from the pool */
    assert_eq!(denom, "mTSLA");
    assert_eq!(recipient, "provider");
    assert_eq!(amount, Uint128::new(100_000_000));

    /* Check our LP position after someone else deposits 100 mTSLA */
    let bin = &query(deps.as_ref(), mock_env(), QueryMsg::LiquidityPosition {
        address: Addr::unchecked("provider"),
    }).unwrap();
    let res: LiquidityPositionResponse = from_binary(&bin).unwrap();
    let position: ProviderPosition = res.position;

    /* Check that our LP share is zero */
    assert_eq!(position.asset_pool_partial_share, Uint128::zero());
    /* Assert other funds have not been touched */
    assert_eq!(position.asset_pool_total_share, Uint128::new(100_000_000));

    /* Verify pool state was updated after our withdrawal */
    let res = query(deps.as_ref(), mock_env(), QueryMsg::PoolState{ }).unwrap();
    let pool_state: PoolStateResponse = from_binary(&res).unwrap();
    assert_eq!(pool_state.assets_in_reserve, Uint128::new(100_000_000));
    assert_eq!(pool_state.total_asset_pool_share, Uint128::new(100_000_000));

}
