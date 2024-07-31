use std::process::Command;

use crate::{
    cli::Verbosity, error::Error, handler::Handler, request::contract_build::ContractBuildRequest, response::{contract_build::ContractBuildResponse, Response}};

impl Handler for ContractBuildRequest {
    type Error = Error;
    type Response = Response;

    fn handle(self, _verbosity: Verbosity, mock_sgx: bool) -> Result<Self::Response, Self::Error> {
        let mut cargo = Command::new("cargo");
        let command = cargo
            .arg("wasm")
            .args(["--manifest-path", &self.manifest_path.display().to_string()])
            .env("RUSTFLAGS", "-C link-arg=-s");
        
        if mock_sgx {
            command.arg("--features=mock-sgx");
        }

        println!("🚧 Building contract binary ...");
        let output = command.output().map_err(|e| Error::GenericErr(format!("{}", e.to_string())))?;
        if !output.status.success() {
            return Err(Error::GenericErr(format!("Couldn't build contract. \n{:?}", output)));
        }
    
        Ok(ContractBuildResponse.into())
    }
}
