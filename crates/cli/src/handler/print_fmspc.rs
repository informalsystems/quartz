use async_trait::async_trait;
use color_eyre::{eyre::eyre, owo_colors::OwoColorize, Report, Result};
use tracing::info;

use crate::{
    config::Config,
    handler::Handler,
    request::print_fmspc::PrintFmspcRequest,
    response::{print_fmspc::PrintFmspcResponse, Response},
};

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

        info!("{}", "\nGenerating dummy quote".blue().bold());

        Ok(PrintFmspcResponse.into())
    }
}
