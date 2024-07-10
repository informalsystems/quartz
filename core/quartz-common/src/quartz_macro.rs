#[cfg(feature = "macro")]
#[macro_export]
macro_rules! quartz_server {
    ($server_type:ty, $service_new:expr) => {
        quartz_server!($crate::cli::Cli, $server_type, $service_new);
    };
    ($cli_type:ty, $server_type:ty, $service_new:expr) => {
        use std::{
            sync::{Arc, Mutex},
            time::Duration,
        };

        use clap::Parser;
        use $crate::{
            contract::state::{Config, LightClientOpts},
            enclave::{
                attestor::{Attestor, EpidAttestor},
                server::CoreService,
            },
            proto::core_server::CoreServer,
        };
        use tonic::transport::Server;

        #[tokio::main(flavor = "current_thread")]
        async fn main() -> Result<(), Box<dyn std::error::Error>> {
            let args = <$cli_type>::parse();

            let light_client_opts = LightClientOpts::new(
                args.chain_id,
                args.trusted_height.into(),
                Vec::from(args.trusted_hash)
                    .try_into()
                    .expect("invalid trusted hash"),
                (
                    args.trust_threshold.numerator(),
                    args.trust_threshold.denominator(),
                ),
                args.trusting_period,
                args.max_clock_drift,
                args.max_block_lag,
            )?;

            let config = Config::new(
                EpidAttestor.mr_enclave()?,
                Duration::from_secs(30 * 24 * 60),
                light_client_opts,
            );

            let sk = Arc::new(Mutex::new(None));

            Server::builder()
                .add_service(CoreServer::new(CoreService::new(
                    config,
                    sk.clone(),
                    EpidAttestor,
                )))
                .add_service($service_new(sk.clone()))
                .serve(args.rpc_addr)
                .await?;

            Ok(())
        }
    };
}
