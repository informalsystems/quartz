use cosmwasm_std::{Addr, Int128};
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub enum ExecuteMsg {
    // OpenCreditLine {},
    DrawCredit {
        receiver: Addr,
        amount: Int128,
    },
    DrawCreditFromTender {
        debtor: Addr,
        amount: Int128,
    },
    TransferCreditFromTender {
        sender: Addr,
        receiver: Addr,
        amount: Int128,
    },
    IncreaseBalance {
        receiver: Addr,
        amount: Int128,
    },
    DecreaseBalance {
        receiver: Addr,
        amount: Int128,
    },
    Lock {},
    Unlock {},
    AddOwner {
        new: Addr,
    },
}