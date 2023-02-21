use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{
    coins, to_binary, Addr, Binary, BlockInfo, Coin, CosmosMsg, Decimal, Empty,
    Response, StdResult, Timestamp, Uint128, WasmMsg,
};
use cw20::{BalanceResponse, Cw20Coin, Cw20ExecuteMsg, Cw20QueryMsg};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};
use hopers_bet::fast_oracle::{
    msg::ExecuteMsg as FastOracleExecuteMsg,
    msg::InstantiateMsg as FastOracleInstantiateMsg,
    msg::QueryMsg as FastOracleQueryMsg,
};
use hopers_bet::price_prediction::{
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    response::{ConfigResponse, StatusResponse},
    Config,
};
use hopers_bet::price_prediction::{Direction, FinishedRound, WalletInfo};

use cw20_base::msg::InstantiateMsg as Cw20InstantiateMsg;

// use std::borrow::BorrowMut;
use std::convert::TryInto;
// use std::ops::Add;

use crate::state::{MyGameResponse, PendingRewardResponse};

fn mock_app() -> App {
    App::default()
}

pub fn contract_price_prediction() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

pub fn contract_fast_oracle() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        |deps, _, _info, msg: FastOracleExecuteMsg| -> StdResult<Response> {
            match msg {
                FastOracleExecuteMsg::Update { price } => {
                    deps.storage.set(b"price", &price.to_be_bytes());
                    Ok(Response::default())
                }
                FastOracleExecuteMsg::Owner { owner: _ } => todo!(),
            }
        },
        |deps, _, _, _: FastOracleInstantiateMsg| -> StdResult<Response> {
            deps.storage
                .set(b"price", &Uint128::new(1_000_000u128).to_be_bytes());
            Ok(Response::default())
        },
        |deps, _, msg: FastOracleQueryMsg| -> StdResult<Binary> {
            match msg {
                FastOracleQueryMsg::Price {} => {
                    let res = deps.storage.get(b"price").unwrap_or_default();
                    let price = Uint128::from(u128::from_be_bytes(
                        res.as_slice().try_into().unwrap(),
                    ));

                    to_binary(&price)
                }
            }
        },
    );
    Box::new(contract)
}

pub fn contract_cw20() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    );
    Box::new(contract)
}

fn update_price(
    router: &mut App,
    config: ConfigResponse,
    price: Uint128,
    sender: &Addr,
) {
    let update_price_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.fast_oracle_addr.to_string(),
        msg: to_binary(&FastOracleExecuteMsg::Update { price }).unwrap(),
        funds: vec![],
    });

    router
        .execute_multi(sender.clone(), [update_price_msg].to_vec())
        .unwrap();
}

fn start_next_round(
    router: &mut App,
    prediction_market_addr: &Addr,
    sender: &Addr,
) {
    router.update_block(|block| {
        block.time = block.time.plus_seconds(600);
        block.height += 1;
    });

    let start_live_round_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: prediction_market_addr.to_string(),
        msg: to_binary(&ExecuteMsg::CloseRound {}).unwrap(),
        funds: vec![],
    });

    router
        .execute_multi(sender.clone(), [start_live_round_msg].to_vec())
        .unwrap();
}

fn init_fast_oracle_contract(router: &mut App, owner: &Addr) -> Addr {
    // println!("prediction_market_code_id, {:?}", prediction_market_code_id);

    let msg = FastOracleInstantiateMsg {};

    let fast_oracle_code_id = router.store_code(contract_fast_oracle());

    router
        .instantiate_contract(
            fast_oracle_code_id,
            Addr::unchecked("owner"),
            &msg,
            &[],
            "fast_oracle",
            Some(owner.to_string()),
        )
        .unwrap()
}

fn init_cw20_Contract(router: &mut App, owner: &Addr) -> Addr {
    // println!("prediction_market_code_id, {:?}", prediction_market_code_id);

    let msg = Cw20InstantiateMsg {
        name: "Hopers".to_string(),
        symbol: "Hopers".to_string(),
        decimals: 6,
        initial_balances: vec![
            Cw20Coin {
                address: "user1".to_string(),
                amount: Uint128::new(1000),
            },
            Cw20Coin {
                address: "user2".to_string(),
                amount: Uint128::new(1000),
            },
            Cw20Coin {
                address: "user3".to_string(),
                amount: Uint128::new(1000),
            },
            Cw20Coin {
                address: "user4".to_string(),
                amount: Uint128::new(1000),
            },
        ],
        mint: None,
        marketing: None,
    };

    let cw20_code_id = router.store_code(contract_cw20());

    router
        .instantiate_contract(
            cw20_code_id,
            Addr::unchecked("owner"),
            &msg,
            &[],
            "fast_oracle",
            Some(owner.to_string()),
        )
        .unwrap()
}

fn create_prediction_market(
    router: &mut App,
    owner: &Addr,
    config: Config,
) -> Addr {
    let prediction_market_code_id =
        router.store_code(contract_price_prediction());

    router.set_block(BlockInfo {
        height: 0,
        time: Timestamp::from_seconds(0),
        chain_id: "testing".to_string(),
    });

    let mut msg = InstantiateMsg {
        config: config.clone(),
    };

    let fast_oracle_addr: Addr = init_fast_oracle_contract(router, owner);
    let cw20_addr: Addr = init_cw20_Contract(router, owner);

    msg.config.fast_oracle_addr = fast_oracle_addr;
    msg.config.token_addr = cw20_addr;

    router
        .instantiate_contract(
            prediction_market_code_id,
            owner.clone(),
            &msg,
            &[],
            "prediction_market",
            Some(owner.to_string()),
        )
        .unwrap()
}

fn execute_bet(
    router: &mut App,
    user: Addr,
    amount: Uint128,
    direction: Direction,
    token_addr: &Addr,
    prediction_market_addr: &Addr,
    round_id: Uint128,
) {
    let increase_allowance_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_addr.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::IncreaseAllowance {
            spender: prediction_market_addr.to_string(),
            amount,
            expires: None,
        })
        .unwrap(),
        funds: vec![],
    });
    let bet_msg: CosmosMsg;
    match direction {
        Direction::Bear => {
            bet_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: prediction_market_addr.to_string(),
                msg: to_binary(&ExecuteMsg::BetBear { amount, round_id })
                    .unwrap(),
                funds: vec![],
            });
        }
        Direction::Bull => {
            bet_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: prediction_market_addr.to_string(),
                msg: to_binary(&ExecuteMsg::BetBull { amount, round_id })
                    .unwrap(),
                funds: vec![],
            });
        }
    }

    router
        .execute_multi(user, [increase_allowance_msg, bet_msg].to_vec())
        .unwrap();
}

#[test]

fn test_bet() {
    let mut router = mock_app();
    let owner = Addr::unchecked("owner");

    let default_config: Config = Config {
        next_round_seconds: Uint128::new(600u128),
        fast_oracle_addr: Addr::unchecked("fast_oracle"),
        minimum_bet: Uint128::new(1u128),
        burn_fee: Uint128::new(100u128),
        gaming_fee: Uint128::new(200u128),
        token_addr: Addr::unchecked("token_contract"),
    };

    let prediction_market_addr =
        create_prediction_market(&mut router, &owner, default_config.clone());

    start_next_round(&mut router, &prediction_market_addr, &owner);

    let config: ConfigResponse = router
        .wrap()
        .query_wasm_smart(
            prediction_market_addr.to_string(),
            &QueryMsg::Config {},
        )
        .unwrap();

    execute_bet(
        &mut router,
        Addr::unchecked("user1"),
        Uint128::new(100),
        Direction::Bear,
        &config.token_addr,
        &prediction_market_addr,
        Uint128::zero(),
    );

    execute_bet(
        &mut router,
        Addr::unchecked("user2"),
        Uint128::new(50),
        Direction::Bull,
        &config.token_addr,
        &prediction_market_addr,
        Uint128::zero(),
    );

    //-----------------------------------------------------close the round and check the pending reward of first user-------------------------------------------

    // update_price(&mut router, config, price, sender)
    start_next_round(&mut router, &prediction_market_addr, &owner);
    update_price(&mut router, config, Uint128::new(100000), &owner);
    start_next_round(&mut router, &prediction_market_addr, &owner);

    let status: StatusResponse = router
        .wrap()
        .query_wasm_smart(prediction_market_addr.clone(), &QueryMsg::Status {})
        .unwrap();
    // println!("status {:?}", status);

    let config: ConfigResponse = router
        .wrap()
        .query_wasm_smart(
            prediction_market_addr.to_string(),
            &QueryMsg::Config {},
        )
        .unwrap();

    execute_bet(
        &mut router,
        Addr::unchecked("user1"),
        Uint128::new(100),
        Direction::Bear,
        &config.token_addr,
        &prediction_market_addr,
        Uint128::new(2),
    );

    execute_bet(
        &mut router,
        Addr::unchecked("user2"),
        Uint128::new(50),
        Direction::Bull,
        &config.token_addr,
        &prediction_market_addr,
        Uint128::new(2),
    );

    //-----------------------------------------------------------Test second user bet to check pending reward-------------------------------------------------

    start_next_round(&mut router, &prediction_market_addr, &owner);
    update_price(&mut router, config, Uint128::new(200000), &owner);
    start_next_round(&mut router, &prediction_market_addr, &owner);

    let pending_reward_user1: PendingRewardResponse = router
        .wrap()
        .query_wasm_smart(
            prediction_market_addr.clone(),
            &QueryMsg::MyPendingReward {
                player: Addr::unchecked("user1"),
            },
        )
        .unwrap();
    let pending_reward_user2: PendingRewardResponse = router
        .wrap()
        .query_wasm_smart(
            prediction_market_addr.clone(),
            &QueryMsg::MyPendingReward {
                player: Addr::unchecked("user2"),
            },
        )
        .unwrap();

    println!(
        "pending reward for user1{:?}, pending reward for user2 {:?}",
        pending_reward_user1, pending_reward_user2
    );

    //---------------------------------------------------Test Claim ----------------------------------------------------------------------//

    let config: ConfigResponse = router
        .wrap()
        .query_wasm_smart(
            prediction_market_addr.to_string(),
            &QueryMsg::Config {},
        )
        .unwrap();

    let claim_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: prediction_market_addr.to_string(),
        msg: to_binary(&ExecuteMsg::CollectWinnings {}).unwrap(),
        funds: vec![],
    });

    router
        .execute_multi(Addr::unchecked("user1"), [claim_msg].to_vec())
        .unwrap();

    let claim_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: prediction_market_addr.to_string(),
        msg: to_binary(&ExecuteMsg::CollectWinnings {}).unwrap(),
        funds: vec![],
    });

    router
        .execute_multi(Addr::unchecked("user2"), [claim_msg].to_vec())
        .unwrap();

    //----------------------------------------------check balance after the claim----------------------------------------------

    let user1_balance: BalanceResponse = router
        .wrap()
        .query_wasm_smart(
            config.token_addr.to_string(),
            &Cw20QueryMsg::Balance {
                address: "user1".to_string(),
            },
        )
        .unwrap();

    let user2_balance: BalanceResponse = router
        .wrap()
        .query_wasm_smart(
            config.token_addr.to_string(),
            &Cw20QueryMsg::Balance {
                address: "user2".to_string(),
            },
        )
        .unwrap();

    let contract_balance: BalanceResponse = router
        .wrap()
        .query_wasm_smart(
            config.token_addr.to_string(),
            &Cw20QueryMsg::Balance {
                address: prediction_market_addr.to_string(),
            },
        )
        .unwrap();

    println!("user1 balance {:?}", user1_balance);
    println!("user2 balance {:?}", user2_balance);
    println!("contract balance {:?}", contract_balance);

    //------------------------------------------------Test Distribute Reward--------------------------------------------------------------------//

    let distribute_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: prediction_market_addr.to_string(),
        msg: to_binary(&ExecuteMsg::DistributeFund {
            dev_wallet_list: vec![
                WalletInfo {
                    address: Addr::unchecked("admin1"),
                    ratio: Decimal::from_ratio(50 as u128, 100 as u128),
                },
                WalletInfo {
                    address: Addr::unchecked("admin2"),
                    ratio: Decimal::from_ratio(50 as u128, 100 as u128),
                },
            ],
        })
        .unwrap(),
        funds: vec![],
    });

    router
        .execute_multi(Addr::unchecked("owner"), [distribute_msg].to_vec())
        .unwrap();

    let admin1_balance: BalanceResponse = router
        .wrap()
        .query_wasm_smart(
            config.token_addr.to_string(),
            &Cw20QueryMsg::Balance {
                address: "admin1".to_string(),
            },
        )
        .unwrap();

    let admin2_balance: BalanceResponse = router
        .wrap()
        .query_wasm_smart(
            config.token_addr.to_string(),
            &Cw20QueryMsg::Balance {
                address: "admin2".to_string(),
            },
        )
        .unwrap();

    let contract_balance: BalanceResponse = router
        .wrap()
        .query_wasm_smart(
            config.token_addr.to_string(),
            &Cw20QueryMsg::Balance {
                address: prediction_market_addr.to_string(),
            },
        )
        .unwrap();

    println!("admin1_balance {:?}", admin1_balance);
    println!("admin2 balance {:?}", admin2_balance);
    println!("contract balance {:?}", contract_balance);
}
