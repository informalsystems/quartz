use std::fmt::Debug;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Deps, HexBinary, StdError};

use super::attested::Noop;
use crate::{error::Error, msg::HasDomainType, state::SESSION};

pub type AnySigned<M, P, S> = Signed<M, AnyAuth<P, S>>;
pub type EnclaveSigned<M, S> = Signed<M, EnclaveAuth<S>>;
pub type UserSigned<M, P, S> = Signed<M, UserAuth<P, S>>;

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

pub trait MsgVeifier {
    type PubKey;
    type Sig;

    fn verify(&self, pub_key: Self::PubKey, sig: Self::Sig) -> Result<(), Error>;
}

pub trait Auth<P, S> {
    fn pub_key(&self, deps: Deps<'_>) -> Result<P, Error>;
    fn sig(self) -> S;
}

#[derive(Clone, Debug, PartialEq)]
pub enum AnyAuth<P, S> {
    Enclave(EnclaveAuth<S>),
    User(UserAuth<P, S>),
}

impl<P, S> Auth<P, S> for AnyAuth<P, S>
where
    P: TryFrom<HexBinary> + Clone,
    <P as TryFrom<HexBinary>>::Error: Debug,
{
    fn pub_key(&self, deps: Deps<'_>) -> Result<P, Error> {
        match self {
            Self::Enclave(e) => e.pub_key(deps),
            Self::User(u) => u.pub_key(deps),
        }
    }

    fn sig(self) -> S {
        match self {
            Self::Enclave(e) => Auth::<P, S>::sig(e),
            Self::User(u) => u.sig(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EnclaveAuth<S> {
    pub sig: S,
}

impl<S> EnclaveAuth<S> {
    pub fn new(sig: S) -> Self {
        Self { sig }
    }
}

impl<P, S> Auth<P, S> for EnclaveAuth<S>
where
    P: TryFrom<HexBinary>,
    <P as TryFrom<HexBinary>>::Error: Debug,
{
    fn pub_key(&self, deps: Deps<'_>) -> Result<P, Error> {
        let session = SESSION.load(deps.storage).map_err(Error::Std)?;
        let raw_pub_key = session.pub_key().ok_or(Error::MissingSessionPublicKey)?;
        let pub_key = raw_pub_key
            .try_into()
            .map_err(|e| StdError::generic_err(format!("{e:?}")))?;
        Ok(pub_key)
    }

    fn sig(self) -> S {
        self.sig
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UserAuth<P, S> {
    pub pub_key: P,
    pub sig: S,
}

impl<P, S> UserAuth<P, S> {
    pub fn new(pub_key: P, sig: S) -> Self {
        Self { pub_key, sig }
    }
}

impl<P: Clone, S> Auth<P, S> for UserAuth<P, S> {
    fn pub_key(&self, _deps: Deps<'_>) -> Result<P, Error> {
        Ok(self.pub_key.clone())
    }

    fn sig(self) -> S {
        self.sig
    }
}

impl<M: MsgVeifier> MsgVeifier for Noop<M> {
    type PubKey = M::PubKey;
    type Sig = M::Sig;

    fn verify(&self, pub_key: Self::PubKey, sig: Self::Sig) -> Result<(), Error> {
        self.0.verify(pub_key, sig)
    }
}
