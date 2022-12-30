use anyhow::Result as AnyResult;

use crate::msg::{
    ExecuteMsg, GetConfigResponse, QueryMsg, ReceiveMsg, StakedBalanceAtHeightResponse,
    StakedValueResponse, TotalStakedAtHeightResponse, TotalValueResponse,
};
use crate::ContractError;

use cosmwasm_std::{to_binary, Addr, Uint128};
use cw20::Cw20Coin;
use cw_multi_test::{next_block, App, AppResponse, Contract, ContractWrapper, Executor};
use cw_utils::Duration;

pub const UNBONDING_DURATION: u64 = 86_400 * 14; // two weeks

pub fn store_staking(app: &mut App) -> u64 {
    let contract = Box::new(ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    ));
    app.store_code(contract)
}

pub fn store_cw20(app: &mut App) -> u64 {
    let contract = Box::new(ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    ));
    app.store_code(contract)
}

#[derive(Debug)]
struct SuiteBuilder {
    initial_balances: Vec<Cw20Coin>,
}

impl SuiteBuilder {
    pub fn new() -> Self {
        Self {
            initial_balances: vec![],
        }
    }

    #[track_caller]
    pub fn build(self) -> Suite {
        let mut app = App::default();
        let owner = Addr::unchecked("owner");

        let cw20_code_id = store_cw20(&mut app);
        let token_contract = app
            .instantiate_contract(
                cw20_code_id,
                owner.clone(),
                &cw20_base::msg::InstantiateMsg {
                    name: String::from("Test"),
                    symbol: String::from("TEST"),
                    decimals: 6,
                    initial_balances: self.initial_balances,
                    mint: None,
                    marketing: None,
                },
                &[],
                "cw20-token",
                Some(owner.to_string()),
            )
            .unwrap();
        app.update_block(next_block);

        let staking_code_id = store_staking(&mut app);
        let staking_contract = app
            .instantiate_contract(
                staking_code_id,
                owner.clone(),
                &crate::msg::InstantiateMsg {
                    owner: Some(owner.to_string()),
                    manager: Some("manager".to_string()),
                    token_address: token_contract.to_string(),
                    unstaking_duration: Some(Duration::Time(UNBONDING_DURATION)),
                },
                &[],
                "staking",
                Some(owner.to_string()),
            )
            .unwrap();
        app.update_block(next_block);

        Suite {
            owner,
            app,
            token_contract,
            staking_contract,
        }
    }
}

pub struct Suite {
    pub owner: Addr,
    pub app: App,
    pub token_contract: Addr,
    pub staking_contract: Addr,
}
