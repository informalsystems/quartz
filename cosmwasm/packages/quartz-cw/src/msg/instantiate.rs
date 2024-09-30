use cosmwasm_schema::cw_serde;
use cosmwasm_std::StdError;
use sha2::{Digest, Sha256};

use crate::{
    msg::{
        execute::attested::{
            Attested, DefaultAttestation, HasUserData, RawAttested, RawDefaultAttestation,
        },
        HasDomainType,
    },
    state::{Config, RawConfig, UserData},
};

#[derive(Clone, Debug, PartialEq)]
pub struct Instantiate<A = DefaultAttestation>(pub Attested<CoreInstantiate, A>);

#[cw_serde]
pub struct RawInstantiate<RA = RawDefaultAttestation>(RawAttested<RawCoreInstantiate, RA>);

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

impl<RA> HasDomainType for RawInstantiate<RA>
where
    RA: HasDomainType,
{
    type DomainType = Instantiate<RA::DomainType>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct CoreInstantiate {
    config: Config,
}

impl CoreInstantiate {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}

#[cw_serde]
pub struct RawCoreInstantiate {
    config: RawConfig,
}

impl TryFrom<RawCoreInstantiate> for CoreInstantiate {
    type Error = StdError;

    fn try_from(value: RawCoreInstantiate) -> Result<Self, Self::Error> {
        Ok(Self {
            config: value.config.try_into()?,
        })
    }
}

impl From<CoreInstantiate> for RawCoreInstantiate {
    fn from(value: CoreInstantiate) -> Self {
        Self {
            config: value.config.into(),
        }
    }
}

impl HasDomainType for RawCoreInstantiate {
    type DomainType = CoreInstantiate;
}

impl HasUserData for CoreInstantiate {
    fn user_data(&self) -> UserData {
        let mut hasher = Sha256::new();
        hasher.update(
            serde_json::to_string(&RawCoreInstantiate::from(self.clone()))
                .expect("infallible serializer"),
        );
        let digest: [u8; 32] = hasher.finalize().into();

        let mut user_data = [0u8; 64];
        user_data[0..32].copy_from_slice(&digest);
        user_data
    }
}
