use std::{convert::Into, default::Default};

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{HexBinary, StdError};
use quartz_tee_ra::{
    intel_sgx::dcap::{Collateral, Quote3, Quote3Error},
    IASReport,
};
use serde::Serialize;

/// Alias for an owned DCAP quote. This is the main part of a DCAP attestation generated by an
/// enclave that we want to verify on-chain.
pub type Quote = Quote3<Vec<u8>>;

#[cfg(not(feature = "mock-sgx"))]
pub type DefaultAttestation = DcapAttestation;
#[cfg(not(feature = "mock-sgx"))]
pub type RawDefaultAttestation = RawDcapAttestation;

#[cfg(feature = "mock-sgx")]
pub type DefaultAttestation = MockAttestation;
#[cfg(feature = "mock-sgx")]
pub type RawDefaultAttestation = RawMockAttestation;

use crate::{
    msg::HasDomainType,
    state::{MrEnclave, UserData},
};

/// A wrapper struct for holding a message and it's attestation.
#[derive(Clone, Debug, PartialEq)]
pub struct Attested<M, A> {
    pub msg: M,
    pub attestation: A,
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

/// A trait that defines how to extract user data from a given type.
pub trait HasUserData {
    fn user_data(&self) -> UserData;
}

/// A verifiable EPID attestation report generated by an enclave.
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

/// A verifiable DCAP attestation generated by an enclave.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DcapAttestation {
    quote: Quote,
    collateral: Collateral,
}

impl DcapAttestation {
    pub fn new(quote: Quote, collateral: Collateral) -> Self {
        Self { quote, collateral }
    }

    pub fn into_tuple(self) -> (Quote, Collateral) {
        (self.quote, self.collateral)
    }
}

#[cw_serde]
pub struct RawDcapAttestation {
    pub quote: HexBinary,
    pub collateral: serde_json::Value,
}

impl TryFrom<RawDcapAttestation> for DcapAttestation {
    type Error = StdError;

    fn try_from(value: RawDcapAttestation) -> Result<Self, Self::Error> {
        let quote_bytes: Vec<u8> = value.quote.into();
        let quote = quote_bytes
            .try_into()
            .map_err(|e: Quote3Error| StdError::parse_err("Quote", e.to_string()))?;
        let collateral = serde_json::from_value(value.collateral)
            .map_err(|e| StdError::parse_err("Collateral", e.to_string()))?;

        Ok(Self { quote, collateral })
    }
}

impl From<DcapAttestation> for RawDcapAttestation {
    fn from(value: DcapAttestation) -> Self {
        Self {
            quote: value.quote.as_ref().to_vec().into(),
            collateral: serde_json::to_vec(&value.collateral)
                .expect("infallible serializer")
                .into(),
        }
    }
}

impl HasDomainType for RawDcapAttestation {
    type DomainType = DcapAttestation;
}

impl HasUserData for DcapAttestation {
    fn user_data(&self) -> UserData {
        let report_data = self.quote.app_report_body().report_data();
        let report_data_slice: &[u8] = report_data.as_ref();
        report_data_slice
            .to_owned()
            .try_into()
            .expect("fixed size array")
    }
}

impl Attestation for DcapAttestation {
    fn mr_enclave(&self) -> MrEnclave {
        let mr_enclave = self.quote.app_report_body().mr_enclave();
        let mr_enclave_slice: &[u8] = mr_enclave.as_ref();
        mr_enclave_slice
            .to_owned()
            .try_into()
            .expect("fixed size array")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MockAttestation(pub UserData);

impl Default for MockAttestation {
    fn default() -> Self {
        Self([0u8; 64])
    }
}

#[cw_serde]
pub struct RawMockAttestation(pub HexBinary);

impl TryFrom<RawMockAttestation> for MockAttestation {
    type Error = StdError;

    fn try_from(value: RawMockAttestation) -> Result<Self, Self::Error> {
        Ok(Self(value.0.to_array()?))
    }
}

impl From<MockAttestation> for RawMockAttestation {
    fn from(value: MockAttestation) -> Self {
        Self(value.0.into())
    }
}

impl HasDomainType for RawMockAttestation {
    type DomainType = MockAttestation;
}

impl HasUserData for MockAttestation {
    fn user_data(&self) -> UserData {
        self.0
    }
}

impl Attestation for MockAttestation {
    fn mr_enclave(&self) -> MrEnclave {
        Default::default()
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
