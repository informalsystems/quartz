use cosmwasm_schema::cw_serde;
use cosmwasm_std::{HexBinary, StdError};
use sha2::{Digest, Sha256};

use crate::msg::execute::attested::{
    Attested, EpidAttestation, HasUserData, RawAttested, RawEpidAttestation,
};
use crate::msg::HasDomainType;
use crate::state::{MrEnclave, UserData};

#[derive(Clone, Debug, PartialEq)]
pub struct Instantiate<A = EpidAttestation>(pub(crate) Attested<CoreInstantiate, A>);

#[cw_serde]
pub struct RawInstantiate<RA = RawEpidAttestation>(RawAttested<RawCoreInstantiate, RA>);

impl<RA> TryFrom<RawInstantiate<RA>> for Instantiate<RA::DomainType>
where
    RA: HasDomainType,
{
    type Error = StdError;

    fn try_from(value: RawInstantiate<RA>) -> Result<Self, Self::Error> {
        Ok(Self(TryFrom::try_from(value.0)?))
    }
}

impl<RA> From<Instantiate<RA::DomainType>> for RawInstantiate<RA>
where
    RA: HasDomainType,
{
    fn from(value: Instantiate<RA::DomainType>) -> Self {
        Self(From::from(value.0))
    }
}

impl HasDomainType for RawInstantiate {
    type DomainType = Instantiate;
}

#[derive(Clone, Debug, PartialEq)]
pub struct CoreInstantiate {
    mr_enclave: MrEnclave,
    // TODO(hu55a1n1): config - e.g. Epoch duration, light client opts
}

impl CoreInstantiate {
    pub fn mr_enclave(&self) -> MrEnclave {
        self.mr_enclave
    }
}

#[cw_serde]
pub struct RawCoreInstantiate {
    mr_enclave: HexBinary,
}

impl TryFrom<RawCoreInstantiate> for CoreInstantiate {
    type Error = StdError;

    fn try_from(value: RawCoreInstantiate) -> Result<Self, Self::Error> {
        let mr_enclave = value.mr_enclave.to_array()?;
        Ok(Self { mr_enclave })
    }
}

impl From<CoreInstantiate> for RawCoreInstantiate {
    fn from(value: CoreInstantiate) -> Self {
        Self {
            mr_enclave: value.mr_enclave.into(),
        }
    }
}

impl HasDomainType for RawCoreInstantiate {
    type DomainType = CoreInstantiate;
}

impl HasUserData for CoreInstantiate {
    fn user_data(&self) -> UserData {
        let mut hasher = Sha256::new();
        hasher.update(self.mr_enclave);
        let digest: [u8; 32] = hasher.finalize().into();

        let mut user_data = [0u8; 64];
        user_data[0..32].copy_from_slice(&digest);
        user_data
    }
}
