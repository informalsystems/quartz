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
                "testing".to_string(),
                "--trusted-height".to_string(),
                trusted_height.to_string(),
                "--trusted-hash".to_string(),
                trusted_hash.to_string(),
            ];

            let res = run_enclave(
                enclave_dir.join("Cargo.toml").display().to_string(),
                config.mock_sgx,
                enclave_args,
            )
            .await?;
        }

        // set cwd to enclave app
        env::set_current_dir(enclave_dir).map_err(|e| Error::GenericErr(e.to_string()))?;

        // gramine private key
        gramine_sgx_gen_private_key().await?;
        // gramine manifest
        gramine_manifest(&trusted_height.to_string(), &trusted_hash.to_string()).await?;
        // gramine sign
        gramine_sgx_sign().await?;
        // run quartz enclave
        gramine_sgx().await?;

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
    // Change directory to DIR_QUARTZ_ENCLAVE
    println!("... gen priv key if it doesn't exist");

    // Launch the gramine-sgx-gen-private-key command
    let output = Command::new("gramine-sgx-gen-private-key")
        .output()
        .await
        .map_err(|e| {
            Error::GenericErr(format!(
                "Failed to execute gramine-sgx-gen-private-key: {}",
                e
            ))
        })?;

    // Check if the command succeeded
    if !output.status.success() {
        return Err(Error::GenericErr(format!(
            "Couldn't generate gramine priv key. {:?}",
            output.stderr
        )));
    }

    Ok(())
}

async fn gramine_manifest(trusted_height: &str, trusted_hash: &str) -> Result<(), Error> {
    let current_dir = env::current_dir().map_err(|e| Error::GenericErr(e.to_string()))?;

    // let gcc_output = Command::new("gcc")
    //     .arg("-dumpmachine")
    //     .output()
    //     .map_err(|e| Error::GenericErr(format!("Failed to execute gcc -dumpmachine: {e}")))?;
    // let arch_libdir = format!("/lib/{}", String::from_utf8_lossy(&gcc_output.stdout).trim());
    let arch_libdir = format!("/lib/{}", target_lexicon::HOST);
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
        .arg(format!(
            "-Dquartz_dir={}",
            current_dir.display().to_string()
        ))
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
