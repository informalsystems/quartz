use cosmwasm_std::{
    coins,
    testing::{message_info, mock_dependencies, mock_env},
    Addr,
};
use transfers_contract::{
    contract::execute,
    msg::execute::Request,
    state::{DENOM, REQUESTS},
};

#[test]
fn test_deposit() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("user1"), &coins(100, "token"));

    DENOM
        .save(deps.as_mut().storage, &"token".to_string())
        .unwrap();
    REQUESTS.save(deps.as_mut().storage, &Vec::new()).unwrap();

    let response = execute::deposit(deps.as_mut(), env.clone(), info.clone()).unwrap();

    assert_eq!(response.events.len(), 1);
    assert_eq!(response.events[0].ty, "transfer");
    assert_eq!(response.events[0].attributes[0].value, "user");

    let requests = REQUESTS.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        requests,
        vec![Request::Deposit(info.sender.clone(), 100u128.into())]
    );
}

#[test]
fn test_withdraw() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("user1"), &[]);

    DENOM
        .save(deps.as_mut().storage, &"token".to_string())
        .unwrap();
    REQUESTS.save(deps.as_mut().storage, &Vec::new()).unwrap();

    let response = execute::withdraw(deps.as_mut(), env.clone(), info.clone()).unwrap();

    assert_eq!(response.events.len(), 1);
    assert_eq!(response.events[0].ty, "transfer");
    assert_eq!(response.events[0].attributes[0].value, "user");

    let requests = REQUESTS.load(deps.as_ref().storage).unwrap();
    assert_eq!(requests, vec![Request::Withdraw(info.sender.clone())]);
}
