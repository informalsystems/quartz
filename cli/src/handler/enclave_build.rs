use std::process::Command;

use crate::{
    cli::Verbosity, error::Error, handler::Handler, request::enclave_build::EnclaveBuildRequest, response::{enclave_build::EnclaveBuildResponse, Response}};

impl Handler for EnclaveBuildRequest {
    type Error = Error;
    type Response = Response;

    fn handle(self, _verbosity: Verbosity) -> Result<Self::Response, Self::Error> {
        // TODO: mock-sgx flag
        let target_dir = self.directory.join("enclave/target");
        let mut cargo = Command::new("cargo");
        let command = cargo
            .args(["build", "--release"])
            .env("CARGO_TARGET_DIR", target_dir);

        // if features, add arg
        println!("🚧 Building enclave binary ...");
        let output = command.output().map_err(|e| Error::GenericErr(format!("here: {}", e.to_string())))?;
        if !output.status.success() {
            return Err(Error::GenericErr(format!("Couldn't build enclave. {:?}", output)));
        }
    
        Ok(EnclaveBuildResponse.into())
    }
}
