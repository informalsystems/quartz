use std::env;

use async_trait::async_trait;
use tokio::{process::Command, sync::watch};
use tracing::{debug, info};

use crate::{
    config::Config,
    error::Error,
    handler::{utils::helpers::get_hash_height, Handler},
    request::enclave_start::EnclaveStartRequest,
    response::{enclave_start::EnclaveStartResponse, Response},
};

#[async_trait]
impl Handler for EnclaveStartRequest {
    type Error = Error;
    type Response = Response;

    async fn handle(self, mut config: Config) -> Result<Self::Response, Self::Error> {
        // Get trusted height and hash
        let (trusted_height, trusted_hash) = get_hash_height(self.use_latest_trusted, &mut config)?;

        let enclave_dir = config.app_dir.join("enclave");

        if config.mock_sgx {
            let enclave_args: Vec<String> = vec![
                "--chain-id".to_string(),
                config.chain_id.to_string(),
                "--trusted-height".to_string(),
                trusted_height.to_string(),
                "--trusted-hash".to_string(),
                trusted_hash.to_string(),
            ];

            // Run quartz enclave and block
            let _res = run_mock_enclave(
                enclave_dir.join("Cargo.toml").display().to_string(),
                config.mock_sgx,
                enclave_args,
                self.shutdown_rx,
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

async fn run_mock_enclave(
    manifest_path: String,
    mock_sgx: bool,
    enclave_args: Vec<String>,
    shutdown_rx: Option<watch::Receiver<()>>,
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
    let mut child = command
        .spawn()
        .map_err(|e| Error::GenericErr(e.to_string()))?;

    match shutdown_rx {
        Some(mut rx) => {
            tokio::select! {
                status = child.wait() => {
                    handle_child_status(status.map_err(|e| Error::GenericErr(e.to_string()))?)?;
                }
                _ = rx.changed() => {
                    println!("Shutdown signal received. Terminating enclave...");
                    let _ = child.kill().await;
                }
            }
        }
        None => {
            // If no shutdown receiver is provided, just wait for the child process
            let status = child
                .wait()
                .await
                .map_err(|e| Error::GenericErr(e.to_string()))?;
            handle_child_status(status)?;
        }
    }

    Ok(())
}

fn handle_child_status(status: std::process::ExitStatus) -> Result<(), Error> {
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
