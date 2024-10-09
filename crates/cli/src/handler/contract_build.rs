use std::process::Command;

use async_trait::async_trait;
use color_eyre::{eyre::eyre, owo_colors::OwoColorize, Report, Result};
use tracing::{debug, info};

use crate::{
    config::Config,
    handler::Handler,
    request::contract_build::ContractBuildRequest,
    response::{contract_build::ContractBuildResponse, Response},
};

#[async_trait]
impl Handler for ContractBuildRequest {
    type Response = Response;

    async fn handle<C: AsRef<Config> + Send>(self, config: C) -> Result<Self::Response, Report> {
        let config = config.as_ref();
        info!("{}", "\nPeforming Contract Build".blue().bold());

        let mut cargo = Command::new("cargo");
        let command = cargo
            .arg("build")
            .arg("--release")
            .args(["--target", "wasm32-unknown-unknown"])
            .arg("--lib")
            .args([
                "--target-dir",
                &config.app_dir.join("target").display().to_string(),
            ])
            .args([
                "--manifest-path",
                &self.contract_manifest.display().to_string(),
            ])
            .env("RUSTFLAGS", "-C link-arg=-s");

        if config.mock_sgx {
            debug!("Building with mock-sgx enabled");
            command.arg("--features=mock-sgx");
        }

        info!("{}", "🚧 Building contract binary ...".green().bold());
        let status = command.status()?;

        if !status.success() {
            return Err(eyre!("Couldn't build contract. \n{:?}", status));
        }

        config.log_build(false).await?;

        Ok(ContractBuildResponse.into())
    }
}
