use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
use crate::testing::mock_querier::{mock_dependencies, OwnedMockDeps};
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{
    coins, from_binary, to_binary, Addr, CosmosMsg, Response, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use leveraged_pools::pool::{
    Cw20HookMsg, ExecuteMsg, HyperparametersResponse, InstantiateMsg,
    LiquidityPositionResponse, PoolStateResponse, PriceHistoryResponse,
    ProtocolRatioResponse, ProviderPosition, QueryMsg,
};

/* Create a 2x pool from a CW20
 * + TS liquidity at 1000:1 mTSLA:UST
 * + Minimum protocol ratio 2.5
 * + Rebalance ratio 2.0
 * + 0.5% premium on minting 2x assets
 * + 10% premium on rebalanced positions
 */
fn mtsla_ust_2x_init(deps: &mut OwnedMockDeps) -> Response {
    /* Create a TerraSwap pool and fill it with mTSLA and uusd */
    deps.querier.with_terraswap_pools(&[(
        &"mTSLA-UST".to_string(),
        (
            &"uusd".to_string(),
            &Uint128::from(1_000_000_000_000u128),
            &"mTSLA".to_string(),
            &Uint128::from(1_000_000_000u128),
        ),
    )]);

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
    let msg = QueryMsg::Hyperparameters {};
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
    let msg = QueryMsg::PoolState {};
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    let pool_state: PoolStateResponse = from_binary(&res).unwrap();
    assert_eq!(pool_state.assets_in_reserve, Uint128::zero());

    /* Assert that inital price was correctly queried from mocked TerraSwap */
    assert_eq!(pool_state.opening_snapshot.timestamp > 0, true);
    assert_eq!(
        pool_state.opening_snapshot.asset_price.u128() / 1_000_000,
        1_000
    );

    /* Query asset price history */
    let msg = QueryMsg::PriceHistory {};
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    let u_price_history: PriceHistoryResponse = from_binary(&res).unwrap();
    let genesis_snapshot = u_price_history.price_history;
    assert_eq!(genesis_snapshot.len(), 1);

    let genesis_snapshot = genesis_snapshot[0];
    /* At genesis leveraged price should equal asset price */
    assert_eq!(
        genesis_snapshot.asset_price,
        genesis_snapshot.leveraged_price
    );
    /* Verify genesis snapshot price is correct */
    assert_eq!(genesis_snapshot.asset_price.u128() / 1_000_000, 1_000);
}

#[test]
fn proper_mint() {
    let mut deps = mock_dependencies(&[]);

    /* mTSLA pool init */
    mtsla_ust_2x_init(&mut deps);

    /* Provide 100 mTSLA as liquidity */
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "provider".to_string(),
        amount: Uint128::new(100_000_000),
        msg: to_binary(&Cw20HookMsg::ProvideLiquidity {}).unwrap(),
    });
    execute(deps.as_mut(), mock_env(), mock_info("mTSLA", &[]), msg).unwrap();

    /*
     * Attempt to mint a leveraged position with 100mTSLA which should fail as
     * protocol-ratio would fall below the minimum
     */
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "minter".to_string(),
        amount: Uint128::new(100_000_000),
        msg: to_binary(&Cw20HookMsg::MintLeveragedPosition {}).unwrap(),
    });
    match execute(deps.as_mut(), mock_env(), mock_info("mTSLA", &[]), msg) {
        Err(e) => match e {
            ContractError::WouldViolatePoolHealth {} => {}
            _ => panic!("Expected WouldViolatePoolHealth but found {}", e),
        },
        _ => panic!("Pool did not enforce PR while minting"),
    }

    /*
     * Attempt to mint a leveraged position which would make a PR of 2.499999
     * and therefore should be rejected too
     */
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "minter".to_string(),
        amount: Uint128::new(66_666_667),
        msg: to_binary(&Cw20HookMsg::MintLeveragedPosition {}).unwrap(),
    });
    match execute(deps.as_mut(), mock_env(), mock_info("mTSLA", &[]), msg) {
        Err(e) => match e {
            ContractError::WouldViolatePoolHealth {} => {}
            _ => panic!("Expected WouldViolatePoolHealth but found {}", e),
        },
        _ => panic!("Pool did not enforce PR while minting"),
    }

    /*
     * Finally mint a position which creates a legal PR of 2.500000
     */
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "minter".to_string(),
        amount: Uint128::new(66_666_666),
        msg: to_binary(&Cw20HookMsg::MintLeveragedPosition {}).unwrap(),
    });
    execute(deps.as_mut(), mock_env(), mock_info("mTSLA", &[]), msg).unwrap();

    /* Verify legal PR of 2.5 */
    let bin =
        &query(deps.as_ref(), mock_env(), QueryMsg::ProtocolRatio {}).unwrap();
    let res: ProtocolRatioResponse = from_binary(&bin).unwrap();
    assert_eq!(res.pr, Uint128::new(2_500_000));

    /*
     * Attempt to remove liquidity which should result in an illegal PR
     */
    let msg = ExecuteMsg::WithdrawLiquidity {
        share_of_pool: Uint128::new(100_000_000),
    };
    match execute(deps.as_mut(), mock_env(), mock_info("provider", &[]), msg) {
        Ok(_) => panic!("LP withdrawal was able to create unhealthy PR"),
        Err(_) => {}
    }

    /*
     * I PUT THE LIQUIDITY IN
     */
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "provider".to_string(),
        amount: Uint128::new(100_000_000),
        msg: to_binary(&Cw20HookMsg::ProvideLiquidity {}).unwrap(),
    });
    execute(deps.as_mut(), mock_env(), mock_info("mTSLA", &[]), msg).unwrap();

    /*
     * JUST TO TAKE IT BACK OUT AGAIN
     */
    let msg = ExecuteMsg::WithdrawLiquidity {
        share_of_pool: Uint128::new(100_000_000),
    };
    execute(deps.as_mut(), mock_env(), mock_info("provider", &[]), msg)
        .unwrap();

    /* Verify legal PR of 2.5 */
    let bin =
        &query(deps.as_ref(), mock_env(), QueryMsg::ProtocolRatio {}).unwrap();
    let res: ProtocolRatioResponse = from_binary(&bin).unwrap();
    assert_eq!(res.pr, Uint128::new(2_500_000));

    /*
     * Try to create an illegal PR by taking just a little out
     */
    let msg = ExecuteMsg::WithdrawLiquidity {
        share_of_pool: Uint128::new(100),
    };
    match execute(deps.as_mut(), mock_env(), mock_info("provider", &[]), msg) {
        Ok(_) => panic!("LP withdrawal was able to create unhealthy PR"),
        Err(_) => {}
    }
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
        msg: to_binary(&Cw20HookMsg::ProvideLiquidity {}).unwrap(),
    });
    execute(deps.as_mut(), mock_env(), mock_info("mTSLA", &[]), msg).unwrap();

    /* Verify 100 mTSLA were recorded as pool liquidity */
    let res = query(deps.as_ref(), mock_env(), QueryMsg::PoolState {}).unwrap();
    let pool_state: PoolStateResponse = from_binary(&res).unwrap();
    assert_eq!(pool_state.assets_in_reserve, Uint128::new(100_000_000));
    assert_eq!(pool_state.total_asset_pool_share, Uint128::new(100_000_000));

    /* Verify the pool recorded our position */
    let bin = &query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LiquidityPosition {
            address: Addr::unchecked("provider"),
        },
    )
    .unwrap();
    let res: LiquidityPositionResponse = from_binary(&bin).unwrap();
    let position: ProviderPosition = res.position;

    /* We own the entire pool of course */
    assert_eq!(position.asset_pool_total_share, Uint128::new(100_000_000));
    assert_eq!(position.asset_pool_partial_share, Uint128::new(100_000_000));

    /* Someone else provides 100 mTSLA as well */
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "someone_else".to_string(),
        amount: Uint128::new(100_000_000),
        msg: to_binary(&Cw20HookMsg::ProvideLiquidity {}).unwrap(),
    });
    execute(deps.as_mut(), mock_env(), mock_info("mTSLA", &[]), msg).unwrap();

    /* Check our LP position after someone else deposits 100 mTSLA */
    let bin = &query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LiquidityPosition {
            address: Addr::unchecked("provider"),
        },
    )
    .unwrap();
    let res: LiquidityPositionResponse = from_binary(&bin).unwrap();
    let position: ProviderPosition = res.position;

    /* Assert that we now only own half the pool */
    assert_eq!(position.asset_pool_total_share, Uint128::new(200_000_000));
    assert_eq!(position.asset_pool_partial_share, Uint128::new(100_000_000));

    /* Attempt to withdraw an excessive amount of liquidity */
    let msg = ExecuteMsg::WithdrawLiquidity {
        share_of_pool: Uint128::new(1_000_000_000),
    };
    match execute(deps.as_mut(), mock_env(), mock_info("provider", &[]), msg) {
        Ok(_) => panic!("LP was able to withdraw more than their share!"),
        Err(_) => {}
    }

    /* Attempt to withdraw our liquidity */
    let msg = ExecuteMsg::WithdrawLiquidity {
        share_of_pool: Uint128::new(100_000_000),
    };
    let res =
        execute(deps.as_mut(), mock_env(), mock_info("provider", &[]), msg)
            .unwrap();

    /* Extract Cw20ExecuteMsg::Transfer from response */
    let (denom, receipt) = match &res.messages[0].msg {
        CosmosMsg::Wasm(w) => match w {
            WasmMsg::Execute {
                contract_addr, msg, ..
            } => (contract_addr, msg),
            _ => panic!("Invalid WithdrawLiquidity response"),
        },
        _ => panic!("Invalid WithdrawLiquidity response"),
    };
    let receipt_msg: Cw20ExecuteMsg = from_binary(&receipt).unwrap();
    let (recipient, amount) = match receipt_msg {
        Cw20ExecuteMsg::Transfer { recipient, amount } => (recipient, amount),
        _ => panic!("Invalid WithdrawLiquidity response"),
    };

    /* Assert that we are credited our funds we withdrew from the pool */
    assert_eq!(denom, "mTSLA");
    assert_eq!(recipient, "provider");
    assert_eq!(amount, Uint128::new(100_000_000));

    /* Check our LP position after someone else deposits 100 mTSLA */
    let bin = &query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LiquidityPosition {
            address: Addr::unchecked("provider"),
        },
    )
    .unwrap();
    let res: LiquidityPositionResponse = from_binary(&bin).unwrap();
    let position: ProviderPosition = res.position;

    /* Check that our LP share is zero */
    assert_eq!(position.asset_pool_partial_share, Uint128::zero());
    /* Assert other funds have not been touched */
    assert_eq!(position.asset_pool_total_share, Uint128::new(100_000_000));

    /* Verify pool state was updated after our withdrawal */
    let res = query(deps.as_ref(), mock_env(), QueryMsg::PoolState {}).unwrap();
    let pool_state: PoolStateResponse = from_binary(&res).unwrap();
    assert_eq!(pool_state.assets_in_reserve, Uint128::new(100_000_000));
    assert_eq!(pool_state.total_asset_pool_share, Uint128::new(100_000_000));
}

#[test]
fn price_history() {
    let mut deps = mock_dependencies(&[]);
    let mut env = mock_env();

    /* mTSLA pool init */
    mtsla_ust_2x_init(&mut deps);

    /* Triggering event for price history update */
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "provider".to_string(),
        amount: Uint128::new(100_000_000),
        msg: to_binary(&Cw20HookMsg::ProvideLiquidity {}).unwrap(),
    });
    execute(deps.as_mut(), env.clone(), mock_info("mTSLA", &[]), msg).unwrap();

    /* Check price history */
    let bin =
        &query(deps.as_ref(), env.clone(), QueryMsg::PriceHistory {}).unwrap();
    let res: PriceHistoryResponse = from_binary(&bin).unwrap();
    let history = res.price_history;

    /* Only history is the opening snapshot */
    assert_eq!(history.len(), 1);

    /* Check that we don't update too frequently */
    env.block.time = env.block.time.plus_seconds(1);

    /* Triggering event for price history update */
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "provider".to_string(),
        amount: Uint128::new(100_000_000),
        msg: to_binary(&Cw20HookMsg::ProvideLiquidity {}).unwrap(),
    });
    execute(deps.as_mut(), env.clone(), mock_info("mTSLA", &[]), msg).unwrap();

    /* Check price history */
    let bin =
        &query(deps.as_ref(), env.clone(), QueryMsg::PriceHistory {}).unwrap();
    let res: PriceHistoryResponse = from_binary(&bin).unwrap();
    let history = res.price_history;

    /* Still only 1 price point - not enough time has elapsed */
    assert_eq!(history.len(), 1);

    /* Advance 15 minutes s/t price should be updated */
    env.block.time = env.block.time.plus_seconds(15 * 60);

    /* Triggering event for price history update */
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "provider".to_string(),
        amount: Uint128::new(100_000_000),
        msg: to_binary(&Cw20HookMsg::ProvideLiquidity {}).unwrap(),
    });
    execute(deps.as_mut(), env.clone(), mock_info("mTSLA", &[]), msg).unwrap();

    /* Check price history */
    let bin =
        &query(deps.as_ref(), env.clone(), QueryMsg::PriceHistory {}).unwrap();
    let res: PriceHistoryResponse = from_binary(&bin).unwrap();
    let history = res.price_history;

    /* 2 price points after sufficent time elapsed */
    assert_eq!(history.len(), 2);

    /* Price should not have changed */
    assert_eq!(history[0].asset_price, Uint128::new(1_000_000_000));
    assert_eq!(history[0].leveraged_price, Uint128::new(1_000_000_000));
    assert_eq!(history[1].asset_price, Uint128::new(1_000_000_000));
    assert_eq!(history[1].leveraged_price, Uint128::new(1_000_000_000));
}

#[test]
fn reset_leverage() {
    let mut deps = mock_dependencies(&[]);
    let mut env = mock_env();

    /* mTSLA pool init */
    mtsla_ust_2x_init(&mut deps);

    /* Triggering event for leverage reset */
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "provider".to_string(),
        amount: Uint128::new(100_000_000),
        msg: to_binary(&Cw20HookMsg::ProvideLiquidity {}).unwrap(),
    });
    execute(deps.as_mut(), env.clone(), mock_info("mTSLA", &[]), msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::PoolState {}).unwrap();
    let pool_state: PoolStateResponse = from_binary(&res).unwrap();
    assert_eq!(
        pool_state.opening_snapshot.timestamp,
        env.block.time.seconds()
    );

    /* Ensure we do not set leverage too often */
    env.block.time = env.block.time.plus_seconds(1);

    /* Triggering event for leverage reset */
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "provider".to_string(),
        amount: Uint128::new(100_000_000),
        msg: to_binary(&Cw20HookMsg::ProvideLiquidity {}).unwrap(),
    });
    execute(deps.as_mut(), env.clone(), mock_info("mTSLA", &[]), msg).unwrap();

    /* Query pool state for last leverage reset time */
    let res = query(deps.as_ref(), mock_env(), QueryMsg::PoolState {}).unwrap();
    let pool_state: PoolStateResponse = from_binary(&res).unwrap();

    /* Should not update - too soon */
    assert_ne!(
        pool_state.opening_snapshot.timestamp,
        env.block.time.seconds()
    );

    /* Wait a day and make sure leverage has reset propery */
    env.block.time = env.block.time.plus_seconds(24 * 60 * 60);

    /* Triggering event for leverage reset */
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "provider".to_string(),
        amount: Uint128::new(100_000_000),
        msg: to_binary(&Cw20HookMsg::ProvideLiquidity {}).unwrap(),
    });
    execute(deps.as_mut(), env.clone(), mock_info("mTSLA", &[]), msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::PoolState {}).unwrap();
    let pool_state: PoolStateResponse = from_binary(&res).unwrap();

    /* Should update - leverage has expired */
    assert_eq!(
        pool_state.opening_snapshot.timestamp,
        env.block.time.seconds()
    );
}
