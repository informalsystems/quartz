#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(
    clippy::checked_conversions,
    clippy::panic,
    clippy::panic_in_result_fn,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    rust_2018_idioms,
    unused_lifetimes,
    unused_import_braces,
    unused_qualifications
)]

mod cli;
mod mtcs_server;
mod proto;

use cli::Cli;
use mtcs_server::MtcsService;
use proto::clearing_server::ClearingServer as MtcsServer;
use quartz_common::quartz_server;

// Passing a custom clap Cli is optional
quartz_server!(Cli, MtcsServer, |sk| MtcsServer::new(MtcsService::new(
    sk,
    EpidAttestor
)));

// With default Cli:
// quartz_server!(MtcsServer, |sk| MtcsServer::new(MtcsService::new(sk, EpidAttestor)));
