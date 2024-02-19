pub mod execute;
pub mod instantiate;
pub mod query;

pub use execute::{Execute as ExecuteMsg, RawExecute as RawExecuteMsg};
pub use instantiate::{Instantiate as InstantiateMsg, RawInstantiate as RawInstantiateMsg};

use cosmwasm_std::StdError;

pub trait HasDomainType: From<Self::DomainType> {
    type DomainType: TryFrom<Self, Error = StdError>;
}
