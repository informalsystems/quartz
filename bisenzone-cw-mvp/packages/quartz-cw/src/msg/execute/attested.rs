use cosmwasm_schema::cw_serde;
use cosmwasm_std::{HexBinary, StdError};
use quartz_tee_ra::IASReport;

use crate::msg::HasDomainType;
use crate::state::{MrEnclave, UserData};

#[derive(Clone, Debug, PartialEq)]
pub struct Attested<M, A> {
    msg: M,
    attestation: A,
}

impl<M, A> Attested<M, A> {
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
    mr_enclave: MrEnclave,
    user_data: UserData,
}

impl EpidAttestation {
    pub fn into_tuple(self) -> (IASReport, MrEnclave, UserData) {
        let EpidAttestation {
            report,
            mr_enclave,
            user_data,
        } = self;
        (report, mr_enclave, user_data)
    }

    pub fn report(&self) -> &IASReport {
        &self.report
    }
}

#[cw_serde]
pub struct RawEpidAttestation {
    report: IASReport,
    mr_enclave: HexBinary,
    user_data: HexBinary,
}

impl TryFrom<RawEpidAttestation> for EpidAttestation {
    type Error = StdError;

    fn try_from(value: RawEpidAttestation) -> Result<Self, Self::Error> {
        let mr_enclave = value.mr_enclave.to_array()?;
        let user_data = value.user_data.to_array()?;
        Ok(Self {
            report: value.report,
            mr_enclave,
            user_data,
        })
    }
}

impl From<EpidAttestation> for RawEpidAttestation {
    fn from(value: EpidAttestation) -> Self {
        Self {
            report: value.report,
            mr_enclave: value.mr_enclave.into(),
            user_data: value.user_data.into(),
        }
    }
}

impl HasDomainType for RawEpidAttestation {
    type DomainType = EpidAttestation;
}

impl HasUserData for EpidAttestation {
    fn user_data(&self) -> UserData {
        self.user_data
    }
}

pub trait Attestation {
    fn mr_enclave(&self) -> MrEnclave;
}

impl Attestation for EpidAttestation {
    fn mr_enclave(&self) -> MrEnclave {
        self.report().report.isv_enclave_quote_body.mrenclave()
    }
}
