#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(
    clippy::checked_conversions,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_used,
    trivial_casts,
    trivial_numeric_casts,
    rust_2018_idioms,
    unused_lifetimes,
    unused_import_braces,
    unused_qualifications
)]

pub mod cli;
pub mod error;
pub mod handler;
pub mod request;
pub mod response;

use clap::Parser;
use color_eyre::eyre::Result;
use tracing_subscriber::{util::SubscriberInitExt, EnvFilter};

use crate::{cli::Cli, handler::Handler, request::Request};

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Cli::parse();

    let env_filter = EnvFilter::builder()
        .with_default_directive(args.verbose.to_level_filter().into())
        .from_env_lossy();

    tracing_subscriber::fmt()
        .with_target(false)
        .with_writer(std::io::stderr)
        .with_env_filter(env_filter)
        .finish()
        .init();

    let request = Request::try_from(args.command)?;
    request.handle(args.verbose)?;

    Ok(())
}
