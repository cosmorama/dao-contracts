use crate::helpers::{chain::Chain, helper::create_dao};
use cosmwasm_std::{to_binary, Uint128};
use cw20_stake::{msg::StakedValueResponse, state::Config};
use cw_core_interface::voting::VotingPowerAtHeightResponse;
use test_context::test_context;

// #### ExecuteMsg #####

#[test_context(Chain)]
#[test]
#[ignore]
fn execute_stake_tokens(chain: &mut Chain) {
    let voting_contract = "cw20_staked_balance_voting";

    let res = create_dao(chain, None, "exc_stake_create_dao", chain.user.addr.clone());
    let dao = res.unwrap();

    let voting_addr = dao.state.voting_module.as_str();

    // stake dao tokens:
    chain
        .orc
        .contract_map
        .add_address(voting_contract, voting_addr)
        .unwrap();
    let staking_addr: String = chain
        .orc
        .query(
            voting_contract,
            "exc_stake_q_stake",
            &cw20_staked_balance_voting::msg::QueryMsg::StakingContract {},
        )
        .unwrap()
        .data()
        .unwrap();

    chain
        .orc
        .contract_map
        .add_address("cw20_stake", staking_addr.to_string())
        .unwrap();
    let res = chain
        .orc
        .query(
            "cw20_stake",
            "exc_stake_q_staked_value",
            &cw20_stake::msg::QueryMsg::StakedValue {
                address: chain.user.addr.clone(),
            },
        )
        .unwrap();
    let staked_value: StakedValueResponse = res.data().unwrap();

    assert_eq!(staked_value.value, Uint128::new(0));

    let res = chain
        .orc
        .query(
            "cw20_stake",
            "exc_stake_q_cfg",
            &cw20_stake::msg::QueryMsg::GetConfig {},
        )
        .unwrap();
    let config: Config = res.data().unwrap();

    chain
        .orc
        .contract_map
        .add_address("cw20_base", config.token_address.as_str())
        .unwrap();
    chain
        .orc
        .execute(
            "cw20_base",
            "exc_stake_stake_tokens",
            &cw20_base::msg::ExecuteMsg::Send {
                contract: staking_addr,
                amount: Uint128::new(100),
                msg: to_binary(&cw20_stake::msg::ReceiveMsg::Stake {}).unwrap(),
            },
            &chain.user.key,
        )
        .unwrap();

    let res = chain
        .orc
        .query(
            "cw20_stake",
            "exc_stake_q_stake",
            &cw20_stake::msg::QueryMsg::StakedValue {
                address: chain.user.addr.clone(),
            },
        )
        .unwrap();
    let staked_value: StakedValueResponse = res.data().unwrap();

    assert_eq!(staked_value.value, Uint128::new(100));

    chain.orc.poll_for_n_blocks(1, 20_000).unwrap();

    let res = chain
        .orc
        .query(
            "cw_core",
            "exc_stake_q_power",
            &cw_core::msg::QueryMsg::VotingPowerAtHeight {
                address: chain.user.addr.clone(),
                height: None,
            },
        )
        .unwrap();
    let power: VotingPowerAtHeightResponse = res.data().unwrap();

    assert_eq!(power.power, Uint128::new(100));
}
