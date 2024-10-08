use std::{env, path::PathBuf};

use async_trait::async_trait;
use color_eyre::{
    eyre::{eyre, Context},
    owo_colors::OwoColorize,
    Report, Result,
};
use tempfile::tempdir;
use tokio::{fs::File, io::AsyncWriteExt, process::Command};
use tracing::{debug, info};

use crate::{
    config::Config,
    handler::Handler,
    request::print_fmspc::PrintFmspcRequest,
    response::{print_fmspc::PrintFmspcResponse, Response},
};

const GEN_QUOTE_MANIFEST_TEMPLATE: &str = include_str!("../bin/gen-quote.manifest.template");

#[async_trait]
impl Handler for PrintFmspcRequest {
    type Response = Response;

    async fn handle<C: AsRef<Config> + Send>(self, config: C) -> Result<Self::Response, Report> {
        let config = config.as_ref().clone();

        if config.mock_sgx {
            return Err(eyre!(
                "MOCK_SGX is enabled! print-fmpsc is only available if SGX is enabled"
            ));
        }

        let current_exe_path =
            env::current_exe().context("Failed to get current executable path")?;
        let exe_path_str = current_exe_path.to_string_lossy();

        if exe_path_str.contains("target") {
            // i.e. this isn't a `cargo install` based installation

            info!("{}", "\nBuilding dummy enclave".blue().bold());

            let mut cargo = Command::new("cargo");
            let command = cargo.arg("build");

            if exe_path_str.contains("release") {
                // add the release flag to make sure it's built in the right place
                command.arg("--release");
            }

            let status = command.status().await?;

            if !status.success() {
                return Err(eyre!("Couldn't build enclave. {:?}", status));
            }
        }

        debug!("{}", "\nGenerating SGX private key".blue().bold());

        let _ = Command::new("gramine-sgx-gen-private-key")
            .output()
            .await
            .map_err(|e| eyre!("Failed to execute gramine-sgx-gen-private-key: {}", e))?;

        let host = target_lexicon::HOST;
        let arch_libdir = format!(
            "/lib/{}-{}-{}",
            host.architecture, host.operating_system, host.environment
        );

        let home_dir = dirs::home_dir()
            .ok_or_else(|| eyre!("Home directory not set"))?
            .display()
            .to_string();

        let gen_quote_bin_path = file_path(current_exe_path.clone(), "gen-quote");

        let temp_dir = tempdir()?;
        let temp_dir_path = temp_dir.path();

        let gen_quote_manifest_path = temp_dir_path.join("gen-quote.manifest.template");
        let mut gen_quote_manifest_file = File::create(&gen_quote_manifest_path).await?;
        gen_quote_manifest_file
            .write_all(GEN_QUOTE_MANIFEST_TEMPLATE.as_bytes())
            .await?;

        let status = Command::new("gramine-manifest")
            .arg("-Dlog_level=error")
            .arg(format!("-Dhome={}", home_dir))
            .arg(format!("-Darch_libdir={}", arch_libdir))
            .arg("-Dra_type=dcap")
            .arg("-Dra_client_linkable=1")
            .arg(format!(
                "-Dgen_quote_bin_path={}",
                gen_quote_bin_path.display()
            ))
            .arg(gen_quote_manifest_path)
            .arg("gen-quote.manifest")
            .current_dir(temp_dir_path)
            .status()
            .await
            .map_err(|e| eyre!("Failed to execute gramine-manifest: {}", e))?;

        if !status.success() {
            return Err(eyre!(
                "gramine-manifest command failed with status: {:?}",
                status
            ));
        }

        let status = Command::new("gramine-sgx-sign")
            .arg("--manifest")
            .arg("gen-quote.manifest")
            .arg("--output")
            .arg("gen-quote.manifest.sgx")
            .current_dir(temp_dir_path)
            .status()
            .await
            .map_err(|e| eyre!("Failed to execute gramine-sgx-sign: {}", e))?;

        if !status.success() {
            return Err(eyre!(
                "gramine-sgx-sign command failed with status: {:?}",
                status
            ));
        }

        info!("{}", "\nGenerating dummy quote".blue().bold());

        let mut child = Command::new("gramine-sgx")
            .arg("./gen-quote")
            .kill_on_drop(true)
            .current_dir(temp_dir_path)
            .spawn()
            .map_err(|e| eyre!("Failed to spawn gramine-sgx child process: {}", e))?;

        let status = child.wait().await?;
        if !status.success() {
            return Err(eyre!("Couldn't build enclave. {:?}", status));
        }

        Ok(PrintFmspcResponse.into())
    }
}

fn file_path(mut current_exe_path: PathBuf, file_name: &str) -> PathBuf {
    current_exe_path.pop();
    current_exe_path.push(file_name);
    current_exe_path
}
