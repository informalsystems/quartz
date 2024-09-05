use async_trait::async_trait;
use color_eyre::owo_colors::OwoColorize;
use tokio::process::Command;
use tracing::{debug, info};

use crate::{
    config::Config,
    error::Error,
    handler::Handler,
    request::enclave_build::EnclaveBuildRequest,
    response::{enclave_build::EnclaveBuildResponse, Response},
};

#[async_trait]
impl Handler for EnclaveBuildRequest {
    type Error = Error;
    type Response = Response;

    async fn handle<C: AsRef<Config> + Send>(
        self,
        config: C,
    ) -> Result<Self::Response, Self::Error> {
        let config = config.as_ref();
        info!("{}", "\nPeforming Enclave Build".blue().bold());

        let enclave_dir = config.app_dir.join("enclave");

        let mut cargo = Command::new("cargo");
        let command = cargo
            .arg("build")
            .args(["--target-dir", &config.app_dir.join("target").display().to_string()]) // TODO: Where should this be set to?
            .args(["--manifest-path", &enclave_dir.join("Cargo.toml").display().to_string(),
        ]);

        if config.mock_sgx {
            debug!("Building with mock-sgx enabled");
            command.arg("--features=mock-sgx");
        }

        if config.release {
            debug!("Targetting release");
            command.arg("--release");
        }

        info!("{}", "🚧 Running build command ...".green().bold());
        let status = command
            .status()
            .await
            .map_err(|e| Error::GenericErr(e.to_string()))?;

        if !status.success() {
            return Err(Error::GenericErr(format!(
                "Couldn't build enclave. {:?}",
                status
            )));
        }

        config.log_build(true).await?;

        Ok(EnclaveBuildResponse.into())
    }
}
