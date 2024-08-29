use std::{fs, path::Path};

use async_trait::async_trait;
use cargo_metadata::MetadataCommand;
use color_eyre::owo_colors::OwoColorize;
use tokio::{
    process::{Child, Command},
    sync::watch,
};
use tracing::{debug, info};

use crate::{
    config::Config,
    error::Error,
    handler::{utils::helpers::write_cache_hash_height, Handler},
    request::enclave_start::EnclaveStartRequest,
    response::{enclave_start::EnclaveStartResponse, Response},
};

#[async_trait]
impl Handler for EnclaveStartRequest {
    type Error = Error;
    type Response = Response;

    async fn handle<C: AsRef<Config> + Send>(
        self,
        config: C,
    ) -> Result<Self::Response, Self::Error> {
        let config = config.as_ref().clone();
        info!("{}", "\nPeforming Enclave Start".blue().bold());

        // Get trusted height and hash
        let (trusted_height, trusted_hash) = self.get_hash_height(&config)?;

        write_cache_hash_height(trusted_height, trusted_hash, &config).await?;

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
            let enclave_child =
                create_mock_enclave_child(config.app_dir.as_path(), config.release, enclave_args)
                    .await?;
            handle_process(self.shutdown_rx, enclave_child).await?;
        } else {
            let enclave_dir = fs::canonicalize(config.app_dir.join("enclave"))?;

            // gramine private key
            gramine_sgx_gen_private_key(&enclave_dir).await?;

            // gramine manifest
            let quartz_dir_canon = &enclave_dir.join("..");
            gramine_manifest(
                &trusted_height.to_string(),
                &trusted_hash.to_string(),
                quartz_dir_canon,
                &enclave_dir,
            )
            .await?;

            // gramine sign
            gramine_sgx_sign(&enclave_dir).await?;

            // Run quartz enclave and block
            let enclave_child = create_gramine_sgx_child(&enclave_dir).await?;
            handle_process(self.shutdown_rx, enclave_child).await?;
        }

        Ok(EnclaveStartResponse.into())
    }
}

async fn handle_process(
    shutdown_rx: Option<watch::Receiver<()>>,
    mut child: Child,
) -> Result<(), Error> {
    info!("{}", "Running enclave ...".green().bold());
    match shutdown_rx {
        Some(mut rx) => {
            tokio::select! {
                status = child.wait() => {
                    handle_child_status(status.map_err(|e| Error::GenericErr(e.to_string()))?)?;
                }
                _ = rx.changed() => {
                    info!("Enclave shutdown signal received.");
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

async fn create_mock_enclave_child(
    app_dir: &Path,
    release: bool,
    enclave_args: Vec<String>,
) -> Result<Child, Error> {
    let enclave_dir = app_dir.join("enclave");
    let target_dir = app_dir.join("target");

    // Use the enclave package metadata to get the path to the program binary
    let package_name = MetadataCommand::new()
        .manifest_path(&enclave_dir.join("Cargo.toml"))
        .exec()
        .map_err(|e| Error::GenericErr(e.to_string()))?
        .root_package()
        .ok_or("No root package found in the metadata")
        .map_err(|e| Error::GenericErr(e.to_string()))?
        .name
        .clone();

    let executable = if release {
        target_dir.join("release").join(package_name)
    } else {
        target_dir.join("debug").join(package_name)
    };

    let mut command = Command::new(executable.display().to_string());

    command.args(enclave_args);

    debug!("Enclave Start Command: {:?}", command);

    info!("{}", "🚧 Spawning enclave process ...".green().bold());
    let child = command
        .spawn()
        .map_err(|e| Error::GenericErr(e.to_string()))?;

    Ok(child)
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

async fn gramine_sgx_gen_private_key(enclave_dir: &Path) -> Result<(), Error> {
    // Launch the gramine-sgx-gen-private-key command
    Command::new("gramine-sgx-gen-private-key")
        .current_dir(enclave_dir)
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

async fn gramine_manifest(
    trusted_height: &str,
    trusted_hash: &str,
    quartz_dir: &Path,
    enclave_dir: &Path,
) -> Result<(), Error> {
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
        .arg(format!("-Dquartz_dir={}", quartz_dir.display()))
        .arg(format!("-Dtrusted_height={}", trusted_height))
        .arg(format!("-Dtrusted_hash={}", trusted_hash))
        .arg("quartz.manifest.template")
        .arg("quartz.manifest")
        .current_dir(enclave_dir)
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

async fn gramine_sgx_sign(enclave_dir: &Path) -> Result<(), Error> {
    let status = Command::new("gramine-sgx-sign")
        .arg("--manifest")
        .arg("quartz.manifest")
        .arg("--output")
        .arg("quartz.manifest.sgx")
        .current_dir(enclave_dir)
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

async fn create_gramine_sgx_child(enclave_dir: &Path) -> Result<Child, Error> {
    info!("🚧 Spawning enclave process ...");

    let child = Command::new("gramine-sgx")
        .arg("./quartz")
        .current_dir(enclave_dir)
        .spawn()?;

    Ok(child)
}
