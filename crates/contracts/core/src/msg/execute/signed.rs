use std::fmt::Debug;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::StdError;

use super::attested::Noop;
use crate::{error::Error, msg::HasDomainType};

pub type AnySigned<M, P, S> = Signed<M, AnyAuth<P, S>>;

#[derive(Clone, Debug, PartialEq)]
pub struct Signed<M, A> {
    msg: M,
    auth: A,
}

impl<M, A> Signed<M, A> {
    pub fn new(msg: M, auth: A) -> Self {
        Self { msg, auth }
    }

    pub fn into_tuple(self) -> (M, A) {
        let Self { msg, auth } = self;
        (msg, auth)
    }

    pub fn msg(&self) -> &M {
        &self.msg
    }

    pub fn auth(&self) -> &A {
        &self.auth
    }
}

#[cw_serde]
pub struct RawSigned<RM, RA> {
    pub msg: RM,
    pub auth: RA,
}

impl<RM, RA> RawSigned<RM, RA> {
    pub fn new(msg: RM, auth: RA) -> Self {
        Self { msg, auth }
    }
}

impl<RM, RA> HasDomainType for RawSigned<RM, RA>
where
    RM: HasDomainType,
    RA: HasDomainType,
{
    type DomainType = Signed<RM::DomainType, RA::DomainType>;
}

impl<RM, RA> TryFrom<RawSigned<RM, RA>> for Signed<RM::DomainType, RA::DomainType>
where
    RM: HasDomainType,
    RA: HasDomainType,
{
    type Error = StdError;

    fn try_from(value: RawSigned<RM, RA>) -> Result<Self, Self::Error> {
        Ok(Self {
            msg: value.msg.try_into()?,
            auth: value.auth.try_into()?,
        })
    }
}

impl<RM, RA> From<Signed<RM::DomainType, RA::DomainType>> for RawSigned<RM, RA>
where
    RM: HasDomainType,
    RA: HasDomainType,
{
    fn from(value: Signed<RM::DomainType, RA::DomainType>) -> Self {
        Self {
            msg: value.msg.into(),
            auth: value.auth.into(),
        }
    }
}

pub trait MsgVerifier {
    type PubKey;
    type Sig;

    fn verify(&self, pub_key: &Self::PubKey, sig: &Self::Sig) -> Result<(), Error>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnyAuth<P, S> {
    pub pub_key: P,
    pub sig: S,
}

impl<P, S> AnyAuth<P, S> {
    pub fn new(pub_key: P, sig: S) -> Self {
        Self { pub_key, sig }
    }
}

pub trait Auth<P, S> {
    fn pub_key(&self) -> &P;
    fn sig(&self) -> &S;
}

impl<P, S> Auth<P, S> for AnyAuth<P, S> {
    fn pub_key(&self) -> &P {
        &self.pub_key
    }

    fn sig(&self) -> &S {
        &self.sig
    }
}

impl<M: MsgVerifier> MsgVerifier for Noop<M> {
    type PubKey = M::PubKey;
    type Sig = M::Sig;

    fn verify(&self, pub_key: &Self::PubKey, sig: &Self::Sig) -> Result<(), Error> {
        self.0.verify(pub_key, sig)
    }
}
