use clap::Parser;
use color_eyre::eyre::Result;
use tm_prover::{config::Config, prover::prove};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Config::parse();

    prove(args).await
}
