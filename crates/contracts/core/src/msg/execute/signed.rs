use cosmwasm_schema::cw_serde;
use cosmwasm_std::{HexBinary, StdError};
use decaf377_rdsa::{Signature, SpendAuth, VerificationKey};

use crate::{error::Error, msg::HasDomainType};

#[derive(Clone, Debug, PartialEq)]
pub struct Signed<M, S> {
    msg: M,
    sig: S,
}

impl<M, S> Signed<M, S> {
    pub fn new(msg: M, sig: S) -> Self {
        Self { msg, sig }
    }

    pub fn into_tuple(self) -> (M, S) {
        let Self { msg, sig } = self;
        (msg, sig)
    }

    pub fn msg(&self) -> &M {
        &self.msg
    }

    pub fn sig(&self) -> &S {
        &self.sig
    }
}

#[cw_serde]
pub struct RawSigned<RM, RS> {
    pub msg: RM,
    pub sig: RS,
}

impl<RM, RS> TryFrom<RawSigned<RM, RS>> for Signed<RM::DomainType, RS::DomainType>
where
    RM: HasDomainType,
    RS: HasDomainType,
{
    type Error = StdError;

    fn try_from(value: RawSigned<RM, RS>) -> Result<Self, Self::Error> {
        Ok(Self {
            msg: value.msg.try_into()?,
            sig: value.sig.try_into()?,
        })
    }
}

impl<RM, RS> From<Signed<RM::DomainType, RS::DomainType>> for RawSigned<RM, RS>
where
    RM: HasDomainType,
    RS: HasDomainType,
{
    fn from(value: Signed<RM::DomainType, RS::DomainType>) -> Self {
        Self {
            msg: value.msg.into(),
            sig: value.sig.into(),
        }
    }
}

impl<RM, RS> HasDomainType for RawSigned<RM, RS>
where
    RM: HasDomainType,
    RS: HasDomainType,
{
    type DomainType = Signed<RM::DomainType, RS::DomainType>;
}

pub trait Verifier {
    fn verify(&self, pub_key: HexBinary, msg: impl AsRef<[u8]>) -> Result<(), Error>;
}

impl Verifier for Signature<SpendAuth> {
    fn verify(&self, pub_key: HexBinary, msg: impl AsRef<[u8]>) -> Result<(), Error> {
        let vk: VerificationKey<SpendAuth> = pub_key.as_slice().try_into().map_err(|e| {
            StdError::generic_err(format!("Failed to decode verification key: {e}"))
        })?;

        vk.verify(msg.as_ref(), self)
            .map_err(|e| StdError::generic_err(format!("Failed to verify signature: {e}")))?;

        Ok(())
    }
}
