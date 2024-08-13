use std::env;

use async_trait::async_trait;
use tokio::process::Command;
use tracing::{debug, info};

use super::utils::helpers::read_hash_height;
use crate::{
    error::Error,
    handler::Handler,
    request::enclave_start::EnclaveStartRequest,
    response::{enclave_start::EnclaveStartResponse, Response},
    Config,
};

#[async_trait]
impl Handler for EnclaveStartRequest {
    type Error = Error;
    type Response = Response;

    async fn handle(self, config: Config) -> Result<Self::Response, Self::Error> {
        let enclave_dir = self.app_dir.join("enclave");
        let (trusted_height, trusted_hash) = read_hash_height(self.app_dir.as_path())
            .await
            .map_err(|e| Error::GenericErr(e.to_string()))?;

        if config.mock_sgx {
            let enclave_args: Vec<String> = vec![
                "--chain-id".to_string(),
                self.chain_id,
                "--trusted-height".to_string(),
                trusted_height.to_string(),
                "--trusted-hash".to_string(),
                trusted_hash.to_string(),
            ];

            // Run quartz enclave and block
            let _res = run_enclave(
                enclave_dir.join("Cargo.toml").display().to_string(),
                config.mock_sgx,
                enclave_args,
            )
            .await?;
        } else {
            // set cwd to enclave app
            env::set_current_dir(enclave_dir).map_err(|e| Error::GenericErr(e.to_string()))?;
            // gramine private key
            gramine_sgx_gen_private_key().await?;
            // gramine manifest
            gramine_manifest(&trusted_height.to_string(), &trusted_hash.to_string()).await?;
            // gramine sign
            gramine_sgx_sign().await?;
            // Run quartz enclave and block
            gramine_sgx().await?;
        }
        Ok(EnclaveStartResponse.into())
    }
}

async fn run_enclave(
    manifest_path: String,
    mock_sgx: bool,
    enclave_args: Vec<String>,
) -> Result<(), Error> {
    let mut cargo = Command::new("cargo");
    let command = cargo.args(["run", "--release", "--manifest-path", &manifest_path]);

    if mock_sgx {
        debug!("Running with mock-sgx enabled");
        command.arg("--features=mock-sgx");
    }

    command.arg("--");
    command.args(enclave_args);

    println!("command: {:?}", command);

    info!("🚧 Running enclave ...");
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

    Ok(())
}

async fn gramine_sgx_gen_private_key() -> Result<(), Error> {
    // Launch the gramine-sgx-gen-private-key command
    Command::new("gramine-sgx-gen-private-key")
        .output()
        .await
        .map_err(|e| {
            Error::GenericErr(format!(
                "Failed to execute gramine-sgx-gen-private-key: {}",
                e
            ))
        })?;

    // Continue regardless of error
    // > /dev/null 2>&1 || :  # may fail
    Ok(())
}

async fn gramine_manifest(trusted_height: &str, trusted_hash: &str) -> Result<(), Error> {
    let current_dir = env::current_dir().map_err(|e| Error::GenericErr(e.to_string()))?;

    let host = target_lexicon::HOST;
    let arch_libdir = format!(
        "/lib/{}-{}-{}",
        host.architecture, host.operating_system, host.environment
    );

    let ra_client_spid = "51CAF5A48B450D624AEFE3286D314894";
    let home_dir = dirs::home_dir()
        .ok_or(Error::GenericErr("home dir not set".to_string()))?
        .display()
        .to_string();

    let status = Command::new("gramine-manifest")
        .arg("-Dlog_level=error")
        .arg(format!("-Dhome={}", home_dir))
        .arg(format!("-Darch_libdir={}", arch_libdir))
        .arg("-Dra_type=epid")
        .arg(format!("-Dra_client_spid={}", ra_client_spid))
        .arg("-Dra_client_linkable=1")
        .arg(format!("-Dquartz_dir={}", current_dir.display()))
        .arg(format!("-Dtrusted_height={}", trusted_height))
        .arg(format!("-Dtrusted_hash={}", trusted_hash))
        .arg("quartz.manifest.template")
        .arg("quartz.manifest")
        .status()
        .await
        .map_err(|e| Error::GenericErr(e.to_string()))?;

    if !status.success() {
        return Err(Error::GenericErr(format!(
            "Couldn't run gramine manifest. {:?}",
            status
        )));
    }

    Ok(())
}

async fn gramine_sgx_sign() -> Result<(), Error> {
    let status = Command::new("gramine-sgx-sign")
        .arg("--manifest")
        .arg("quartz.manifest")
        .arg("--output")
        .arg("quartz.manifest.sgx")
        .status()
        .await
        .map_err(|e| Error::GenericErr(e.to_string()))?;

    if !status.success() {
        return Err(Error::GenericErr(format!(
            "gramine-sgx-sign command failed. {:?}",
            status
        )));
    }

    Ok(())
}

async fn gramine_sgx() -> Result<(), Error> {
    let status = Command::new("gramine-sgx")
        .arg("./quartz")
        .status()
        .await
        .map_err(|e| Error::GenericErr(e.to_string()))?;

    if !status.success() {
        return Err(Error::GenericErr(format!(
            "gramine-sgx-sign command failed. {:?}",
            status
        )));
    }

    Ok(())
}
