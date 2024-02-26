use std::{
    fs::{read, File},
    io::{Error as IoError, Write},
};

use quartz_cw::msg::execute::attested::HasUserData;

pub trait Attestor {
    type Error: ToString;

    fn quote(&self, user_data: impl HasUserData) -> Result<Vec<u8>, Self::Error>;
}

#[derive(Clone, PartialEq, Debug)]
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
}
