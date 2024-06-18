use cosmwasm_schema::cw_serde;
use cosmwasm_std::StdError;
use quartz_tee_ra::IASReport;

use crate::{
    msg::HasDomainType,
    state::{MrEnclave, UserData},
};

#[derive(Clone, Debug, PartialEq)]
pub struct Attested<M, A> {
    msg: M,
    attestation: A,
}

impl<M, A> Attested<M, A> {
    pub fn new(msg: M, attestation: A) -> Self {
        Self { msg, attestation }
    }

    pub fn into_tuple(self) -> (M, A) {
        let Attested { msg, attestation } = self;
        (msg, attestation)
    }

    pub fn msg(&self) -> &M {
        &self.msg
    }

    pub fn attestation(&self) -> &A {
        &self.attestation
    }
}

#[cw_serde]
pub struct RawAttested<RM, RA> {
    pub msg: RM,
    pub attestation: RA,
}

impl<RM, RA> TryFrom<RawAttested<RM, RA>> for Attested<RM::DomainType, RA::DomainType>
where
    RM: HasDomainType,
    RA: HasDomainType,
{
    type Error = StdError;

    fn try_from(value: RawAttested<RM, RA>) -> Result<Self, Self::Error> {
        Ok(Self {
            msg: value.msg.try_into()?,
            attestation: value.attestation.try_into()?,
        })
    }
}

impl<RM, RA> From<Attested<RM::DomainType, RA::DomainType>> for RawAttested<RM, RA>
where
    RM: HasDomainType,
    RA: HasDomainType,
{
    fn from(value: Attested<RM::DomainType, RA::DomainType>) -> Self {
        Self {
            msg: value.msg.into(),
            attestation: value.attestation.into(),
        }
    }
}

impl<RM, RA> HasDomainType for RawAttested<RM, RA>
where
    RM: HasDomainType,
    RA: HasDomainType,
{
    type DomainType = Attested<RM::DomainType, RA::DomainType>;
}

pub trait HasUserData {
    fn user_data(&self) -> UserData;
}

#[derive(Clone, Debug, PartialEq)]
pub struct EpidAttestation {
    report: IASReport,
}

impl EpidAttestation {
    pub fn new(report: IASReport) -> Self {
        Self { report }
    }

    pub fn into_report(self) -> IASReport {
        self.report
    }
}

#[cw_serde]
pub struct RawEpidAttestation {
    report: IASReport,
}

impl TryFrom<RawEpidAttestation> for EpidAttestation {
    type Error = StdError;

    fn try_from(value: RawEpidAttestation) -> Result<Self, Self::Error> {
        Ok(Self {
            report: value.report,
        })
    }
}

impl From<EpidAttestation> for RawEpidAttestation {
    fn from(value: EpidAttestation) -> Self {
        Self {
            report: value.report,
        }
    }
}

impl HasDomainType for RawEpidAttestation {
    type DomainType = EpidAttestation;
}

impl HasUserData for EpidAttestation {
    fn user_data(&self) -> UserData {
        self.report.report.isv_enclave_quote_body.user_data()
    }
}

pub trait Attestation {
    fn mr_enclave(&self) -> MrEnclave;
}

impl Attestation for EpidAttestation {
    fn mr_enclave(&self) -> MrEnclave {
        self.report.report.isv_enclave_quote_body.mrenclave()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MockAttestation;

#[cw_serde]
pub struct RawMockAttestation;

impl TryFrom<RawMockAttestation> for MockAttestation {
    type Error = StdError;

    fn try_from(_value: RawMockAttestation) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

impl From<MockAttestation> for RawMockAttestation {
    fn from(_value: MockAttestation) -> Self {
        Self
    }
}

impl HasDomainType for RawMockAttestation {
    type DomainType = MockAttestation;
}

impl HasUserData for MockAttestation {
    fn user_data(&self) -> UserData {
        unimplemented!("MockAttestation handler is a noop")
    }
}

impl Attestation for MockAttestation {
    fn mr_enclave(&self) -> MrEnclave {
        unimplemented!("MockAttestation handler is a noop")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AttestedMsgSansHandler<T>(pub T);

#[cw_serde]
pub struct RawAttestedMsgSansHandler<T>(pub T);

impl<T> HasDomainType for RawAttestedMsgSansHandler<T> {
    type DomainType = AttestedMsgSansHandler<T>;
}

impl<T> HasUserData for AttestedMsgSansHandler<T>
where
    T: HasUserData,
{
    fn user_data(&self) -> UserData {
        self.0.user_data()
    }
}

impl<T> TryFrom<RawAttestedMsgSansHandler<T>> for AttestedMsgSansHandler<T> {
    type Error = StdError;

    fn try_from(value: RawAttestedMsgSansHandler<T>) -> Result<Self, Self::Error> {
        Ok(Self(value.0))
    }
}

impl<T> From<AttestedMsgSansHandler<T>> for RawAttestedMsgSansHandler<T> {
    fn from(value: AttestedMsgSansHandler<T>) -> Self {
        Self(value.0)
    }
}
