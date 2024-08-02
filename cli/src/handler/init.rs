use std::{
    path::{Path, PathBuf},
    process::Command,
};

use tracing::trace;

use crate::{
    cli::Verbosity,
    error::Error,
    handler::Handler,
    request::init::InitRequest,
    response::{init::InitResponse, Response},
};

impl Handler for InitRequest {
    type Error = Error;
    type Response = Response;

    fn handle(self, _verbosity: Verbosity) -> Result<Self::Response, Self::Error> {
        trace!("initializing directory structure...");

        if Path::new(&self.name).iter().count() != 1 {
            return Err(Error::GenericErr("App name contains path".to_string()));
        }

        let cli_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");

        let status = Command::new("cargo")
            .arg("generate")
            .arg("--path")
            .arg(cli_manifest_dir.display().to_string())
            .arg("--name")
            .arg(self.name)
            .arg("apps/transfers")
            .status()
            .map_err(|e| Error::GenericErr(e.to_string()))?;

        // Check the output
        if !status.success() {
            return Err(Error::GenericErr(status.to_string()));
        }

        Ok(InitResponse.into())
    }
}
