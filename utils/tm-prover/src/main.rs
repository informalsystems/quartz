use clap::Parser;
use color_eyre::eyre::Result;
use tm_prover::{config::Config, prover::prove};
use tracing_subscriber::{util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Config::parse();

    let env_filter = EnvFilter::builder()
        .with_default_directive(args.verbose.to_level_filter().into())
        .from_env_lossy();

    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(env_filter)
        .finish()
        .init();

    let proof = prove(args).await?;
    println!("{:?}", proof);

    Ok(())
}
