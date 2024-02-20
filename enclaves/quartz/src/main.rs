#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(
    clippy::checked_conversions,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_used,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    rust_2018_idioms,
    unused_lifetimes,
    unused_import_braces,
    unused_qualifications
)]

mod proto;
mod server;

use tonic::transport::Server;

use crate::{proto::quartz::core_server::CoreServer, server::CoreService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:9090".parse()?;
    let core_service = CoreService::default();

    Server::builder()
        .add_service(CoreServer::new(core_service))
        .serve(addr)
        .await?;

    Ok(())
}
