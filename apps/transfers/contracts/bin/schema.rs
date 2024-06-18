use transfers::msg::{ExecuteMsg, InstantiateMsg};
use cosmwasm_schema::write_api;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
    }
}

