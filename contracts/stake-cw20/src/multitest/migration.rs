use anyhow::Result as AnyResult;
use serde::Serialize;

use crate::msg::MigrateMsg;

use cosmwasm_std::{Addr, Uint128};
use cw20::Cw20Coin;
use cw_multi_test::{next_block, App, AppResponse, ContractWrapper, Executor};
use cw_utils::Duration;

pub const ONE_DAY: u64 = 86_400;

pub fn store_staking(app: &mut App) -> u64 {
    let contract = Box::new(
        ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        )
        .with_migrate_empty(crate::contract::migrate),
    );
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

pub fn store_wyndex_staking(app: &mut App) -> u64 {
    let contract = Box::new(ContractWrapper::new(
        wyndex_stake::contract::execute,
        wyndex_stake::contract::instantiate,
        wyndex_stake::contract::query,
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
                    unstaking_duration: Some(Duration::Time(ONE_DAY * 14)),
                },
                &[],
                "staking",
                Some(owner.to_string()),
            )
            .unwrap();
        app.update_block(next_block);

        let wyndex_stake_code_id = store_wyndex_staking(&mut app);

        Suite {
            owner,
            app,
            token_contract,
            staking_contract,
            wyndex_stake_code_id,
        }
    }
}

pub struct Suite {
    pub owner: Addr,
    pub app: App,
    pub token_contract: Addr,
    pub staking_contract: Addr,
    pub wyndex_stake_code_id: u64,
}

impl Suite {
    pub fn owner(&self) -> Addr {
        self.owner.clone()
    }

    pub fn migrate<T: Serialize>(
        &mut self,
        sender: &str,
        code_id: u64,
        msg: &T,
    ) -> AnyResult<AppResponse> {
        let contract = self.staking_contract.clone();
        self.app
            .migrate_contract(Addr::unchecked(sender), contract, msg, code_id)
    }
}

#[test]
fn wyndex_base_migration() {
    let mut suite = SuiteBuilder::new().build();

    let migrate_msg = MigrateMsg {
        new_admin: Some("wyndex_dao".to_owned()),
        pool_contract: "pool".to_owned(),
        tokens_per_power: Uint128::new(1),
        min_bond: Uint128::new(1),
        unbonding_periods: vec![ONE_DAY * 5, ONE_DAY & 10, ONE_DAY * 14, ONE_DAY * 20],
        max_distributions: 6,
    };

    let owner = suite.owner();
    suite
        .migrate(owner.as_str(), suite.wyndex_stake_code_id, &migrate_msg)
        .unwrap();
}
