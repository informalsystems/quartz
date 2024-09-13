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

pub mod cache;
pub mod cli;
pub mod config;
pub mod error;
pub mod handler;
pub mod request;
pub mod response;

use std::path::PathBuf;

use clap::Parser;
use cli::ToFigment;
use color_eyre::{eyre::Result, owo_colors::OwoColorize};
use config::Config;
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use tracing_subscriber::{util::SubscriberInitExt, EnvFilter};

use crate::{cli::Cli, handler::Handler, request::Request};

const BANNER: &str = r"
 ________       ___  ___      ________      ________     __________    ________     
|\   __  \     |\  \|\  \    |\   __  \    |\   __  \   |\___   ___\  |\_____  \    
\ \  \|\  \    \ \  \\\  \   \ \  \|\  \   \ \  \|\  \  \|___ \  \_|   \|___/  /|   
 \ \  \\\  \    \ \  \\\  \   \ \   __  \   \ \   _  _\      \ \  \        /  / /   
  \ \  \\\  \    \ \  \\\  \   \ \  \ \  \   \ \  \\  \       \ \  \      /  /_/__  
   \ \_____  \    \ \_______\   \ \__\ \__\   \ \__\\ _\       \ \__\    |\________\
    \|___| \__\    \|_______|    \|__|\|__|    \|__|\|__|       \|__|     \|_______|
          \|__|                                                                     
                                                                                    
";

use std::process::Command;

#[tokio::main]
async fn main() -> Result<()> {

    fn get_git_commit_hash() -> String {
        // let output = Command::new("git")
        //     .args(&["rev-parse", "--short", "HEAD"])
        //     .output();
    
        let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output();

        match output {
            Ok(output) if output.status.success() => {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            }
            _ => "unknown".to_string(),
        }
    }


    color_eyre::install()?;

    let git_commit_hash = get_git_commit_hash();
    eprintln!("Git commit hash: {}", git_commit_hash.yellow().bold());

    println!("{}", BANNER.yellow().bold());

    let args: Cli = Cli::parse();
    check_path(&args.app_dir)?;

    let env_filter = EnvFilter::builder()
        .with_default_directive(args.verbose.to_level_filter().into())
        .from_env_lossy();

    tracing_subscriber::fmt()
        .with_target(false)
        .with_writer(std::io::stderr)
        .with_env_filter(env_filter)
        .finish()
        .init();

    tracing::info!("Git commit hash: {}", git_commit_hash);

    let config: Config = Figment::new()
        .merge(Toml::file(
            args.app_dir
                .as_ref()
                .unwrap_or(&PathBuf::from("."))
                .join("quartz.toml"),
        ))
        .merge(Env::prefixed("QUARTZ_"))
        .merge(Serialized::defaults(&args))
        .merge(args.command.to_figment())
        .extract()?;

    let request = Request::try_from(args.command)?;
    let response = request.handle(config).await?;

    println!(
        "{}",
        serde_json::to_string(&response).expect("infallible serializer")
    );

    Ok(())
}

fn check_path(path: &Option<PathBuf>) -> Result<(), error::Error> {
    if let Some(path) = path {
        if !path.is_dir() {
            return Err(error::Error::PathNotDir(format!("{}", path.display())));
        }
    }

    Ok(())
}
