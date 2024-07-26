use clap::Parser;
use color_eyre::eyre::Result;
use tm_prover::{cli::Cli, prover::proof};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Cli::parse();

    proof(args).await
}
