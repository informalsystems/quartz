use serde::Serialize;

use crate::response::{enclave_build::EnclaveBuildResponse, init::InitResponse};

pub mod enclave_build;
pub mod init;

#[derive(Clone, Debug, Serialize)]
pub enum Response {
    Init(InitResponse),
    EnclaveBuild(EnclaveBuildResponse),
}
