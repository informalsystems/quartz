use std::{
    fs::{read, File},
    io::{Error as IoError, Write},
};

use quartz_cw::{
    msg::execute::attested::HasUserData,
    state::{MrEnclave, UserData},
};

#[cfg(not(feature = "mock-sgx"))]
pub type DefaultAttestor = EpidAttestor;

#[cfg(feature = "mock-sgx")]
pub type DefaultAttestor = MockAttestor;

/// The trait defines the interface for generating attestations from within an enclave.
pub trait Attestor {
    type Error: ToString;

    fn quote(&self, user_data: impl HasUserData) -> Result<Vec<u8>, Self::Error>;

    fn mr_enclave(&self) -> Result<MrEnclave, Self::Error>;
}

/// An `Attestor` for generating EPID attestations for Gramine based enclaves.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct EpidAttestor;

impl Attestor for EpidAttestor {
    type Error = IoError;

    fn quote(&self, user_data: impl HasUserData) -> Result<Vec<u8>, Self::Error> {
        let user_data = user_data.user_data();
        let mut user_report_data = File::create("/dev/attestation/user_report_data")?;
        user_report_data.write_all(user_data.as_slice())?;
        user_report_data.flush()?;
        read("/dev/attestation/quote")
    }

    fn mr_enclave(&self) -> Result<MrEnclave, Self::Error> {
        let quote = self.quote(NullUserData)?;
        Ok(quote[112..(112 + 32)]
            .try_into()
            .expect("hardcoded array size"))
    }
}

/// An `Attestor` for generating DCAP attestations for Gramine based enclaves.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct DcapAttestor;

impl Attestor for DcapAttestor {
    type Error = IoError;

    fn quote(&self, user_data: impl HasUserData) -> Result<Vec<u8>, Self::Error> {
        let user_data = user_data.user_data();
        let mut user_report_data = File::create("/dev/attestation/user_report_data")?;
        user_report_data.write_all(user_data.as_slice())?;
        user_report_data.flush()?;
        read("/dev/attestation/quote")
    }

    fn mr_enclave(&self) -> Result<MrEnclave, Self::Error> {
        let quote = self.quote(NullUserData)?;
        Ok(quote[112..(112 + 32)]
            .try_into()
            .expect("hardcoded array size"))
    }
}

/// A mock `Attestor` that creates a quote consisting of just the user report data. (only meant for
/// testing purposes)
#[derive(Clone, PartialEq, Debug, Default)]
pub struct MockAttestor;

impl Attestor for MockAttestor {
    type Error = String;

    fn quote(&self, user_data: impl HasUserData) -> Result<Vec<u8>, Self::Error> {
        let user_data = user_data.user_data();
        Ok(user_data.to_vec())
    }

    fn mr_enclave(&self) -> Result<MrEnclave, Self::Error> {
        Ok(Default::default())
    }
}

struct NullUserData;

impl HasUserData for NullUserData {
    fn user_data(&self) -> UserData {
        [0u8; 64]
    }
}
