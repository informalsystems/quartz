pub mod execute;
pub mod instantiate;
pub mod query;

use cosmwasm_std::StdError;
pub use execute::{Execute as ExecuteMsg, RawExecute as RawExecuteMsg};
pub use instantiate::{Instantiate as InstantiateMsg, RawInstantiate as RawInstantiateMsg};

pub trait HasDomainType: From<Self::DomainType> {
    type DomainType: TryFrom<Self, Error = StdError>;
}
