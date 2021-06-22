use crate::contract::query_short_reward_weight;
use crate::math::{erf_plus_one, Sign};
use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::Decimal;

#[test]
fn short_reward_weight_test() {
    let deps = mock_dependencies(20, &[]);
    let e6 = 1000000u128;
    let e7 = 10000000u128;
    assert_eq!(
        query_short_reward_weight(&deps, Decimal::zero())
            .unwrap()
            .short_reward_weight,
        Decimal::from_ratio(002618u128, e6)
    );
    assert_eq!(
        query_short_reward_weight(&deps, Decimal::percent(1))
            .unwrap()
            .short_reward_weight,
        Decimal::from_ratio(0634168u128, e7),
    );
    assert_eq!(
        query_short_reward_weight(&deps, Decimal::percent(2))
            .unwrap()
            .short_reward_weight,
        Decimal::percent(20)
    );
    assert_eq!(
        query_short_reward_weight(&deps, Decimal::percent(4))
            .unwrap()
            .short_reward_weight,
        Decimal::from_ratio(3908998u128, e7)
    );
    assert_eq!(
        query_short_reward_weight(&deps, Decimal::percent(8))
            .unwrap()
            .short_reward_weight,
        Decimal::percent(40)
    );
    assert_eq!(
        query_short_reward_weight(&deps, Decimal::percent(15))
            .unwrap()
            .short_reward_weight,
        Decimal::percent(40)
    );
}

#[test]
fn erf_plus_one_test() {
    let e6 = 1000000u128;
    let e10 = 10000000000u128;
    assert_eq!(
        erf_plus_one(Sign::Negative, Decimal::from_ratio(21213203435u128, e10)),
        Decimal::zero()
    );
    assert_eq!(
        erf_plus_one(Sign::Negative, Decimal::from_ratio(14142135623u128, e10)),
        Decimal::from_ratio(013090u128, e6)
    );
    assert_eq!(
        erf_plus_one(Sign::Positive, Decimal::zero()),
        Decimal::from_ratio(1000000u128, e6)
    );
    assert_eq!(
        erf_plus_one(Sign::Positive, Decimal::from_ratio(14142135623u128, e10)),
        Decimal::from_ratio(1954499u128, e6)
    );
}
