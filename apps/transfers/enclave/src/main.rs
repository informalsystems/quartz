#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(
    clippy::checked_conversions,
    clippy::panic,
    clippy::panic_in_result_fn,
    trivial_casts,
    trivial_numeric_casts,
    rust_2018_idioms,
    unused_lifetimes,
    unused_import_braces,
    unused_qualifications
)]

pub mod cli;
pub mod proto;
pub mod state;
pub mod transfers_server;

use transfers_server::TransfersService;
use proto::settlement_server::SettlementServer as TransfersServer;

use quartz_common::quartz_server;

quartz_server!(TransfersServer, |sk| TransfersServer::new(TransfersService::<EpidAttestor>::new(sk, EpidAttestor)));
