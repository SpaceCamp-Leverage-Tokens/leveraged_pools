use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{coins, from_binary, Addr, Uint128, Response};
use leveraged_pools::pool::{
    InstantiateMsg, QueryMsg, HyperparametersResponse,
    PriceHistoryResponse, PoolStateResponse
};
use crate::contract::{instantiate, query};
use crate::testing::mock_querier::{mock_dependencies, OwnedMockDeps};

fn mtsla_2x_init(deps: &mut OwnedMockDeps) -> Response {
    /* Hyperparameters */
    let msg = InstantiateMsg {
        leverage_amount: Uint128::new(2_000_000),
        minimum_protocol_ratio: Uint128::new(2_000_000),
        rebalance_ratio: Uint128::new(2_500_000),
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

    let res = mtsla_2x_init(&mut deps);
    assert_eq!(0, res.messages.len());

    /* Query hyperparameters and validate they are as we set them to be */
    let msg = QueryMsg::Hyperparameters { };
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    let hyper_p: HyperparametersResponse = from_binary(&res).unwrap();
    assert_eq!(hyper_p.leverage_amount, Uint128::new(2_000_000));
    assert_eq!(hyper_p.minimum_protocol_ratio, Uint128::new(2_000_000));
    assert_eq!(hyper_p.rebalance_ratio, Uint128::new(2_500_000));
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

