use std::path::PathBuf;

use async_trait::async_trait;
use cargo_generate::{generate, GenerateArgs, TemplatePath, Vcs};
use color_eyre::owo_colors::OwoColorize;
use tokio::fs;

use crate::{
    config::Config,
    error::Error,
    handler::Handler,
    request::init::InitRequest,
    response::{init::InitResponse, Response},
};

#[async_trait]
impl Handler for InitRequest {
    type Error = Error;
    type Response = Response;

    // TODO: Add non-template init method
    async fn handle<C: AsRef<Config> + Send>(
        self,
        config: C,
    ) -> Result<Self::Response, Self::Error> {
        let config = config.as_ref();
        println!("{}", "\nPeforming Init".blue().bold());

        let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");

        let parent = self
            .name
            .parent()
            .map(|p| p.to_path_buf())
            .expect("path already validated");
        fs::create_dir_all(&parent)
            .await
            .map_err(|e| Error::GenericErr(e.to_string()))?;

        let file_name = self
            .name
            .file_name()
            .and_then(|f| f.to_str())
            .expect("path already validated");

        let wasm_pack_args = GenerateArgs {
            name: Some(file_name.to_string()),
            destination: Some(config.app_dir.join(parent)),
            overwrite: true,
            vcs: Some(Vcs::Git),
            template_path: TemplatePath {
                // git: Some("git@github.com:informalsystems/cycles-quartz.git".to_string()), // TODO: replace with public http address when open-sourced
                path: Some(root_dir.join("apps/transfers").display().to_string()),
                ..TemplatePath::default()
            },
            ..GenerateArgs::default()
        };

        let result_dir = generate(wasm_pack_args)
            .expect("something went wrong!")
            .display()
            .to_string();

        println!("\n{}", "It's TEE time.".green().bold());
        Ok(InitResponse { result_dir }.into())
    }
}
