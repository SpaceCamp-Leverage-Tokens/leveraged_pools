use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{coins, from_binary, Addr, Uint128};
use crate::msg::{InstantiateMsg, QueryMsg};
use crate::state::{Hyperparameters, PoolState};
use crate::contract::{instantiate, query};
use crate::testing::mock_querier::{mock_dependencies};

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


    /* Hyperparameters */
    let msg = InstantiateMsg {
        leverage_amount: 2_000_000,
        minimum_protocol_ratio: 2_000_000,
        rebalance_ratio: 2_500_000,
        mint_premium: 0_500_000,
        rebalance_premium: 10_000_000,
        /* Previous terraswap pool */
        terraswap_pair_addr: String::from("mTSLA-UST"),
        /* Contract of the asset that is being leveraged */
        leveraged_asset_addr: String::from("mTSLA"),
    };

    /* Initialize leveraged pool */
    let info = mock_info("leveraged", &coins(1000, "big_ones"));
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    /* Query hyperparameters and validate they are as we set them to be */
    let msg = QueryMsg::Hyperparameters { };
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    let hyper_p: Hyperparameters = from_binary(&res).unwrap();
    assert_eq!(hyper_p.leverage_amount, 2_000_000);
    assert_eq!(hyper_p.minimum_protocol_ratio, 2_000_000);
    assert_eq!(hyper_p.rebalance_ratio, 2_500_000);
    assert_eq!(hyper_p.mint_premium, 0_500_000);
    assert_eq!(hyper_p.rebalance_premium, 10_000_000);
    assert_eq!(hyper_p.terraswap_pair_addr, Addr::unchecked("mTSLA-UST"));
    assert_eq!(hyper_p.leveraged_asset_addr, Addr::unchecked("mTSLA"));

    /* Check that pool state was also initialized correctly */
    let msg = QueryMsg::PoolState { };
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    let pool_state: PoolState = from_binary(&res).unwrap();
    assert_eq!(pool_state.assets_in_reserve, 0);

    /* Assert that inital price was correctly queried from mocked TerraSwap */
    assert_eq!(pool_state.asset_opening_price.timestamp > 0, true);
    assert_eq!(pool_state.asset_opening_price.u_price.u128() / 1_000_000, 1_000);
}
