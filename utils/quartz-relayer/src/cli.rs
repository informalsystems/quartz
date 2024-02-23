use clap::Parser;
use tonic::transport::Endpoint;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// RPC server address
    #[clap(long, default_value = "http://localhost:11090")]
    pub enclave_addr: Endpoint,
}
