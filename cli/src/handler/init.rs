use std::path::PathBuf;

use async_trait::async_trait;
use cargo_generate::{generate, GenerateArgs, TemplatePath, Vcs};
use tracing::trace;

use crate::{
    config::{write_config, Config},
    error::Error,
    handler::Handler,
    request::init::InitRequest,
    response::{init::InitResponse, Response},
};

#[async_trait]
impl Handler for InitRequest {
    type Error = Error;
    type Response = Response;

    async fn handle(self, config: Config) -> Result<Self::Response, Self::Error> {
        trace!("initializing directory structure...");

        let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");

        let wasm_pack_args = GenerateArgs {
            name: Some(self.name.clone()),
            destination: Some(config.app_dir.clone()),
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

        write_config(
            config.app_dir.join(self.name).join("quartz.toml").as_path(),
            &config,
        )
        .await?;

        Ok(InitResponse { result_dir }.into())
    }
}
