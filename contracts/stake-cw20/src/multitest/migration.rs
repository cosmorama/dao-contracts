use crate::msg::{
    ExecuteMsg, GetConfigResponse, QueryMsg, ReceiveMsg, StakedBalanceAtHeightResponse,
    StakedValueResponse, TotalStakedAtHeightResponse, TotalValueResponse,
};
use crate::state::MAX_CLAIMS;
use crate::ContractError;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{to_binary, Addr, Empty, MessageInfo, Uint128};
use cw20::Cw20Coin;
use cw_utils::Duration;

use cw_multi_test::{next_block, App, AppResponse, Contract, ContractWrapper, Executor};

use anyhow::Result as AnyResult;

use cw_controllers::{Claim, ClaimsResponse};
use cw_utils::Expiration::AtHeight;

const ADDR1: &str = "addr0001";
const ADDR2: &str = "addr0002";
const ADDR3: &str = "addr0003";
const ADDR4: &str = "addr0004";

pub fn store_staking(app: &mut App) -> u64 {
    let contract = Box::new(ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    ));
    app.store_code(contract)
}

pub fn store_cw20(app: &mut App) -> Box<dyn Contract<Empty>> {
    let contract = Box::new(ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    ));
    app.store_code(contract)
}

#[derive(Debug)]
struct SuiteBuilder {}

impl SuiteBuilder {
    pub fn new() -> Self {
        Self {}
    }

    #[track_caller]
    pub fn build(self) -> Suite {
        let mut app = App::default();
        let owner = Addr::unchecked("owner");

        let cw20_code_id = store_cw20(&mut app);
        let token_contract = app
            .instantiate_contract(
                cw20_code_id,
                Addr::unchecked(owner),
                &cw20_base::msg::InstantiateMsg {
                    name: String::from("Test"),
                    symbol: String::from("TEST"),
                    decimals: 6,
                    initial_balances,
                    mint: None,
                    marketing: None,
                },
                &[],
                "cw20-token",
                Some(owner),
            )
            .unwrap();
        app.update_block(next_block);

        let staking_code_id = store_staking(&mut app);
        let staking_contract = app
            .instantiate_contract(
                staking_code_id,
                Addr::unchecked(owner),
                &crate::msg::InstantiateMsg {
                    owner: Some(owner.clone()),
                    manager: Some("manager".to_string()),
                    token_address: token_contract.to_string(),
                    unstaking_duration,
                },
                &[],
                "staking",
                Some(owner),
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
    pub owner: String,
    pub app: App,
    pub token_contract: Addr,
    pub staking_contract: Addr,
}

fn get_balance<T: Into<String>, U: Into<String>>(
    app: &App,
    contract_addr: T,
    address: U,
) -> Uint128 {
    let msg = cw20::Cw20QueryMsg::Balance {
        address: address.into(),
    };
    let result: cw20::BalanceResponse = app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
    result.balance
}
