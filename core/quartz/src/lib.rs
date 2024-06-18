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

use std::sync::{Arc, Mutex};

use quartz_cw::state::Config;
use quartz_proto::quartz::core_server::CoreServer;
use tonic::transport::Server;

use crate::{
    attestor::EpidAttestor,
    proto::clearing_server::ClearingServer as MtcsServer,
    server::CoreService,
};

use std::net::SocketAddr;

pub mod attestor;
pub mod cli;
pub mod proto;
pub mod server;