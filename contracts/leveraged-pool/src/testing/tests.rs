use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coins, from_binary};
use crate::msg::{InstantiateMsg, QueryMsg};
use crate::state::{Hyperparameters};
use crate::contract::{instantiate, query};


#[test]
fn proper_init() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        leverage_amount: 2_000_000,
        minimum_protocol_ratio: 2_000_000,
        rebalance_ratio: 2_500_000,
        mint_premium: 0_500_000,
        rebalance_premium: 10_000_000,
        terraswap_pair_addr: String::from("mTSLA-UST"),
        leveraged_asset_addr: String::from("mTSLA"),
    };

    let info = mock_info("leveraged", &coins(1000, "big_ones"));
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let msg = QueryMsg::HyperParameters { };
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    let hyper_p: Hyperparameters = from_binary(&res).unwrap();
    assert_eq!(hyper_p.leverage_amount, 2_000_000);
    assert_eq!(hyper_p.minimum_protocol_ratio, 2_000_000);
    assert_eq!(hyper_p.rebalance_ratio, 2_500_000);
    assert_eq!(hyper_p.mint_premium, 0_500_000);
    assert_eq!(hyper_p.rebalance_premium, 10_000_000);
    assert_eq!(hyper_p.terraswap_pair_addr, String::from("mTSLA-UST"));
    assert_eq!(hyper_p.leveraged_asset_addr, String::from("mTSLA"));
}
