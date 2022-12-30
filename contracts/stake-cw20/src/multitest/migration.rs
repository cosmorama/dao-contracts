use anyhow::Result as AnyResult;
use serde::Serialize;

use crate::msg::{GetConfigResponse, MigrateMsg, QueryMsg};

use cosmwasm_std::{Addr, Uint128};
use cw20::Cw20Coin;
use cw_controllers::AdminResponse;
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
    let contract = Box::new(
        ContractWrapper::new(
            wyndex_stake::contract::execute,
            wyndex_stake::contract::instantiate,
            wyndex_stake::contract::query,
        )
        .with_migrate_empty(wyndex_stake::contract::migrate),
    );
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

mod wyndex_stake_helpers {
    use super::*;
    use serde::Deserialize;

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
    #[serde(rename_all = "snake_case")]
    pub enum WyndexQueryMsg {
        Admin {},
        BondingInfo {},
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
    pub struct BondingPeriodInfo {
        pub unbonding_period: u64,
        pub total_staked: Uint128,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
    pub struct BondingInfoResponse {
        pub bonding: Vec<BondingPeriodInfo>,
    }
}

#[test]
fn wyndex_base_migration() {
    let mut suite = SuiteBuilder::new().build();

    // make sure original staking contract is what it is by querying config
    suite
        .app
        .wrap()
        .query_wasm_smart::<GetConfigResponse>(
            suite.staking_contract.clone(),
            &QueryMsg::GetConfig {},
        )
        .unwrap();

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

    // now the same query won't work, as wyndex stake doesn't have such query
    suite
        .app
        .wrap()
        .query_wasm_smart::<GetConfigResponse>(
            suite.staking_contract.clone(),
            &QueryMsg::GetConfig {},
        )
        .unwrap_err();

    // but query specific for wyndex stake will work
    suite
        .app
        .wrap()
        .query_wasm_smart::<AdminResponse>(
            suite.staking_contract.clone(),
            &wyndex_stake_helpers::WyndexQueryMsg::Admin {},
        )
        .unwrap();
}
