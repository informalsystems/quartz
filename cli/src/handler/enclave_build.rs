use std::process::Command;

use crate::{
    cli::Verbosity, error::Error, handler::Handler, request::enclave_build::EnclaveBuildRequest, response::{enclave_build::EnclaveBuildResponse, Response}};

impl Handler for EnclaveBuildRequest {
    type Error = Error;
    type Response = Response;

    fn handle(self, _verbosity: Verbosity, mock_sgx: bool) -> Result<Self::Response, Self::Error> {
        // TODO: mock-sgx flag
        let mut cargo = Command::new("cargo");
        let command = cargo
            .args(["build", "--release"])
            .args(["--manifest-path", &self.manifest_path.display().to_string()]);

        if mock_sgx {
            command.arg("--features=mock-sgx");
        }

        println!("🚧 Building enclave ...");
        let output = command.output().map_err(|e| Error::GenericErr(e.to_string()))?;
        if !output.status.success() {
            return Err(Error::GenericErr(format!("Couldn't build enclave. {:?}", output)));
        }
    
        Ok(EnclaveBuildResponse.into())
    }
}
