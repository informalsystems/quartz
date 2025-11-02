use cosmwasm_std::{coins, Addr};
use transfers_contract::msg::{execute::Request, ExecuteMsg};

use crate::Context;

#[test]
fn test_multitest_deposit() {
    let mut ctx = Context::new();

    let user = Addr::unchecked("user1");
    let funds = coins(100, &"ucosm".to_string());
    let response = ctx
        .execute_msg(user.clone(), &ExecuteMsg::Deposit, &funds)
        .unwrap();

    let event = response
        .events
        .iter()
        .find(|e| e.ty == "wasm-transfer")
        .unwrap();
    assert_eq!(event.attributes.iter().any(|a| a.value == "user"), true);

    let requests = ctx.query_requests().unwrap();
    assert_eq!(
        requests,
        vec![Request::Deposit(
            user.clone(),
            funds[0].amount.u128().into()
        )]
    );
}

#[test]
fn test_multitest_withdraw() {
    let mut ctx = Context::new();

    let user = Addr::unchecked("user1");
    let response = ctx
        .execute_msg(user.clone(), &ExecuteMsg::Withdraw, &[])
        .unwrap();

    let event = response
        .events
        .iter()
        .find(|e| e.ty == "wasm-transfer")
        .unwrap();
    assert_eq!(event.attributes.iter().any(|a| a.value == "user"), true);

    let requests = ctx.query_requests().unwrap();
    assert_eq!(requests, vec![Request::Withdraw(user.clone())]);
}

#[test]
fn test_multitest_deposit_and_withdraw() {
    let mut ctx = Context::new();

    let denom = "ucosm".to_string();
    let user1 = Addr::unchecked("user1");
    let funds1 = coins(100, &denom);
    ctx.execute_msg(user1.clone(), &ExecuteMsg::Deposit, &funds1)
        .unwrap();

    let user2 = Addr::unchecked("user2");
    let funds2 = coins(200, &denom);
    ctx.execute_msg(user2.clone(), &ExecuteMsg::Deposit, &funds2)
        .unwrap();

    ctx.execute_msg(user1.clone(), &ExecuteMsg::Withdraw, &[])
        .unwrap();

    // Verify requests order
    let requests = ctx.query_requests().unwrap();
    assert_eq!(
        requests,
        vec![
            Request::Deposit(user1.clone(), funds1[0].amount.u128().into()),
            Request::Deposit(user2.clone(), funds2[0].amount.u128().into()),
            Request::Withdraw(user1.clone()),
        ]
    );
}
