pub mod execute;
pub mod instantiate;
pub mod query;

use cosmwasm_std::StdError;
pub use execute::{Execute as ExecuteMsg, RawExecute as RawExecuteMsg};
pub use instantiate::{Instantiate as InstantiateMsg, RawInstantiate as RawInstantiateMsg};
use serde::Serialize;

pub trait HasDomainType: From<Self::DomainType> + Serialize {
    type DomainType: TryFrom<Self, Error = StdError>;
}
