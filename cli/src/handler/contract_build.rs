use std::{env::current_dir, process::Command};

use crate::{
    cli::Verbosity, error::Error, handler::Handler, request::contract_build::ContractBuildRequest, response::{contract_build::ContractBuildResponse, Response}};

impl Handler for ContractBuildRequest {
    type Error = Error;
    type Response = Response;

    fn handle(self, _verbosity: Verbosity) -> Result<Self::Response, Self::Error> {
        // TODO: mock-sgx flag
        let root = current_dir().map_err(|e| Error::GenericErr(e.to_string()))?;
        let cargo_pkg_dir = self.directory.join("/contracts/cw-tee-mtcs");
        
        let mut docker = Command::new("docker");
        let command = docker
            .arg("run")
            .arg("--rm")
            .arg("-v")
            .arg(format!("{}:/code", root.display().to_string()))
            .arg("--mount")
            .arg(format!("type=volume,source={}_cache,target=/code/target", root.display().to_string()))
            .arg("--mount")
            .arg("type=volume,source=registry_cache,target=/usr/local/cargo/registry")
            .arg("cosmwasm/rust-optimizer:0.15.0")
            .arg(cargo_pkg_dir); 

        // if features, add arg
        println!("🚧 Building contract binary ...");
        let output = command.output().map_err(|e| Error::GenericErr(format!("here: {}", e.to_string())))?;
        if !output.status.success() {
            return Err(Error::GenericErr(format!("Couldn't build contract. {:?}", output)));
        }
    
        Ok(ContractBuildResponse.into())
    }
}
