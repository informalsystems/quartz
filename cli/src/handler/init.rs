use std::path::{Path, PathBuf};
use cargo_generate::{generate, GenerateArgs, TemplatePath, Vcs};

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
        
        let wasm_pack_args = GenerateArgs {
            name: Some(self.name),
            vcs: Some(Vcs::Git),
            template_path: TemplatePath {
                // git: Some("git@github.com:informalsystems/cycles-quartz.git".to_string()), // TODO: replace with public http address when open-sourced
                path: Some(cli_manifest_dir.join("apps/transfers").display().to_string()),
                subfolder: Some(String::from("apps/transfers")),
                ..TemplatePath::default()
            },
            ..GenerateArgs::default()
        };
    
        let result_dir = generate(wasm_pack_args).expect("something went wrong!").display().to_string();

        Ok(InitResponse { result_dir }.into())
    }
}
