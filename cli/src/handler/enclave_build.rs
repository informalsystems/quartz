use std::process::Command;

use tracing::{debug, trace};

use crate::{
    error::Error,
    handler::Handler,
    request::enclave_build::EnclaveBuildRequest,
    response::{enclave_build::EnclaveBuildResponse, Response}, Config,
};

impl Handler for EnclaveBuildRequest {
    type Error = Error;
    type Response = Response;

    fn handle(self, config: Config) -> Result<Self::Response, Self::Error> {
        let mut cargo = Command::new("cargo");
        let command = cargo
            .args(["build", "--release"])
            .args(["--manifest-path", &self.manifest_path.display().to_string()]);

        if config.mock_sgx {
            debug!("Building with mock-sgx enabled");
            command.arg("--features=mock-sgx");
        }

        trace!("🚧 Building enclave ...");
        let child = command
            .status()
            .map_err(|e| Error::GenericErr(e.to_string()))?;

        if !child.success() {
            return Err(Error::GenericErr(format!(
                "Couldn't build enclave. {:?}",
                child
            )));
        }

        Ok(EnclaveBuildResponse.into())
    }
}
