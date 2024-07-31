use serde::Serialize;

use crate::response::{init::InitResponse, enclave_build::EnclaveBuildResponse};

pub mod init;
pub mod enclave_build;

#[derive(Clone, Debug, Serialize)]
pub enum Response {
    Init(InitResponse),
    EnclaveBuild(EnclaveBuildResponse)
}
