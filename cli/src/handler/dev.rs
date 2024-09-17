use std::{path::PathBuf, process::exit, time::Duration};

use async_trait::async_trait;
use color_eyre::owo_colors::OwoColorize;
// todo get rid of this?
use miette::{IntoDiagnostic, Result};
use quartz_common::proto::core_client::CoreClient;
use tokio::{
    sync::{mpsc, watch},
    time::sleep,
};
use tracing::{debug, info};
use watchexec::Watchexec;
use watchexec_signals::Signal;

use crate::{
    error::Error,
    handler::{utils::helpers::wasmaddr_to_id, Handler},
    request::{
        contract_build::ContractBuildRequest, contract_deploy::ContractDeployRequest,
        dev::DevRequest, enclave_build::EnclaveBuildRequest, enclave_start::EnclaveStartRequest,
        handshake::HandshakeRequest,
    },
    response::{dev::DevResponse, Response},
    Config, BANNER,
};

#[async_trait]
impl Handler for DevRequest {
    type Error = Error;
    type Response = Response;

    async fn handle<C: AsRef<Config> + Send>(
        self,
        config: C,
    ) -> Result<Self::Response, Self::Error> {
        let config = config.as_ref();
        info!("\nPeforming Dev");

        let (tx, rx) = mpsc::channel::<DevRebuild>(32);
        let _res = tx.send(DevRebuild::Init).await;

        if self.watch {
            tokio::spawn(watcher(tx, config.build_log_dir()?));
        }

        dev_driver(rx, &self, config.clone()).await?;

        Ok(DevResponse.into())
    }
}

#[derive(Debug, Clone)]
enum DevRebuild {
    Init,
    Enclave,
    Contract,
}

async fn dev_driver(
    mut rx: mpsc::Receiver<DevRebuild>,
    args: &DevRequest,
    config: Config,
) -> Result<(), Error> {
    // State
    let mut shutdown_tx: Option<watch::Sender<()>> = None;
    let mut first_enclave_message = true;
    let mut first_contract_message = true;
    let mut contract = String::from("");

    // Shutdown enclave upon interruption
    let shutdown_tx_cpy = shutdown_tx.clone();
    ctrlc::set_handler(move || {
        if let Some(tx) = &shutdown_tx_cpy {
            let _res = tx.send(());
        }

        exit(130)
    })
    .expect("Error setting Ctrl-C handler");

    // Drive
    while let Some(dev) = rx.recv().await {
        match dev {
            DevRebuild::Init => {
                clearscreen::clear()?;
                println!("{}", BANNER.yellow().bold());
                info!("{}", "Launching quartz app...".green().bold());

                // Build enclave
                let enclave_build = EnclaveBuildRequest {};
                enclave_build.handle(&config).await?;

                // Build contract
                let contract_build = ContractBuildRequest {
                    contract_manifest: args.contract_manifest.clone(),
                };
                contract_build.handle(&config).await?;

                // Start enclave in background
                let new_shutdown_tx = spawn_enclave_start(args, &config).await?;

                // Deploy new contract and perform handshake
                let res = deploy_and_handshake(None, args, &config).await;

                // Save resulting contract address or shutdown and return error
                match res {
                    Ok(res_contract) => {
                        // Set state
                        contract = res_contract;
                        shutdown_tx = Some(new_shutdown_tx);

                        info!("{}", "Enclave is listening for requests...".green().bold());
                    }
                    Err(e) => {
                        eprintln!("Error launching quartz app");

                        new_shutdown_tx
                            .send(())
                            .expect("Could not send signal on channel");

                        return Err(e);
                    }
                }
            }
            DevRebuild::Enclave => {
                if first_enclave_message {
                    first_enclave_message = false;

                    continue;
                }
                clearscreen::clear()?;
                println!("{}", BANNER.yellow().bold());
                info!("{}", "Rebuilding Enclave...".green().bold());

                if let Some(shutdown_tx) = shutdown_tx.clone() {
                    let _res = shutdown_tx.send(());
                }

                info!("Waiting 1 second for the enclave to shut down");
                sleep(Duration::from_secs(1)).await;

                let new_shutdown_tx = spawn_enclave_start(args, &config).await?;

                // todo: should not unconditionally deploy here
                let res = deploy_and_handshake(Some(&contract), args, &config).await;

                match res {
                    Ok(res_contract) => {
                        // Set state
                        contract = res_contract;
                        shutdown_tx = Some(new_shutdown_tx);

                        info!("{}", "Enclave is listening for requests...".green().bold());
                    }
                    Err(e) => {
                        eprintln!("Error restarting enclave and handshake");

                        new_shutdown_tx
                            .send(())
                            .expect("Could not send signal on channel");

                        return Err(e);
                    }
                }
            }
            DevRebuild::Contract => {
                if first_contract_message {
                    first_contract_message = false;
                    continue;
                }
                clearscreen::clear()?;
                println!("{}", BANNER.yellow().bold());
                info!("{}", "Rebuilding Contract...".green().bold());

                if let Some(shutdown_tx) = shutdown_tx.clone() {
                    let res = deploy_and_handshake(None, args, &config).await;

                    match res {
                        Ok(res_contract) => contract = res_contract,
                        Err(e) => {
                            eprintln!("Error deploying contract and handshake:");

                            shutdown_tx
                                .send(())
                                .expect("Could not send signal on channel");

                            return Err(e);
                        }
                    }
                } else {
                    return Err(Error::GenericErr(
                        "Attempting to redeploy contract, but enclave isn't running".to_string(),
                    ));
                }

                info!("{}", "Enclave is listening for requests...".green().bold());
            }
        }
    }

    Ok(())
}

// Spawns enclve start in a separate task which runs in the background
async fn spawn_enclave_start(
    args: &DevRequest,
    config: &Config,
) -> Result<watch::Sender<()>, Error> {
    // In separate process, launch the enclave
    let (shutdown_tx, shutdown_rx) = watch::channel(());
    let enclave_start = EnclaveStartRequest {
        shutdown_rx: Some(shutdown_rx),
        unsafe_trust_latest: args.unsafe_trust_latest,
        fmspc: args.fmspc.clone(),
        tcbinfo_contract: args.tcbinfo_contract.clone(),
        dcap_verifier_contract: args.dcap_verifier_contract.clone(),
    };

    let config_cpy = config.clone();

    tokio::spawn(async move {
        let res = enclave_start.handle(config_cpy).await?;

        Ok::<Response, Error>(res)
    });

    Ok(shutdown_tx)
}

// TODO: do not shutdown if cli calls fail, just print
async fn deploy_and_handshake(
    contract: Option<&str>,
    args: &DevRequest,
    config: &Config,
) -> Result<String, Error> {
    info!("Waiting for enclave start to deploy contract and handshake");

    // Wait at most 60 seconds to connect to enclave
    let mut i = 30;
    while CoreClient::connect(format!(
        "{}:{}",
        config.enclave_rpc_addr, config.enclave_rpc_port
    ))
    .await
    .is_err()
    {
        sleep(Duration::from_secs(2)).await;
        i -= 1;

        if i == 0 {
            return Err(Error::GenericErr(
                "Could not connect to enclave".to_string(),
            ));
        }
    }
    // Calls which interact with enclave
    info!("Successfully pinged enclave, enclave is running");

    // Deploy contract IF existing contract wasn't pass into the function
    let contract = if let Some(contract) = contract {
        info!("Contract already deployed, reusing");
        contract.to_string()
    } else {
        info!("Deploying contract");
        // Deploy Contract request
        let contract_deploy = ContractDeployRequest {
            init_msg: args.init_msg.clone(),
            label: args.label.clone(),
            contract_manifest: args.contract_manifest.clone(),
        };
        // Call handler
        let cd_res = contract_deploy.handle(config).await;

        // Return contract address or shutdown enclave & error
        match cd_res {
            Ok(Response::ContractDeploy(res)) => res.contract_addr,
            Err(e) => return Err(e),
            _ => unreachable!("Unexpected response variant"),
        }
    };

    // Run handshake
    info!("Running handshake on contract `{}`", contract);
    let handshake = HandshakeRequest {
        contract: wasmaddr_to_id(&contract).map_err(|_| Error::GenericErr(String::default()))?,
        unsafe_trust_latest: args.unsafe_trust_latest,
    };

    let h_res = handshake.handle(config).await;

    match h_res {
        Ok(Response::Handshake(res)) => {
            info!("Handshake complete: {}", res.pub_key);
        }
        Err(e) => {
            return Err(e);
        }
        _ => unreachable!("Unexpected response variant"),
    };

    Ok(contract)
}

async fn watcher(tx: mpsc::Sender<DevRebuild>, log_dir: PathBuf) -> Result<()> {
    let wx = Watchexec::new_async(move |mut action| {
        let tx = tx.clone();

        Box::new(async move {
            if action.signals().any(|sig| sig == Signal::Interrupt) {
                eprintln!("[Quitting...]");
                action.quit();
                return action;
            }

            // Look for updates to filepaths
            if let Some((path, _)) = action.paths().next() {
                debug!("event triggered on path:\n {:?}", path);
                if path
                    .file_name()
                    .expect("events can't trigger on nonexistent files")
                    .eq("enclave")
                {
                    let _res = tx.send(DevRebuild::Enclave).await;
                } else if path
                    .file_name()
                    .expect("events can't trigger on nonexistent files")
                    .eq("contract")
                {
                    let _res = tx.send(DevRebuild::Contract).await;
                }
            }

            action
        })
    })?;

    // Start the engine
    let main = wx.main();

    // Watch all files in quartz app directory
    // TODO: should create_log_dir be called instead? Just enforce building log in all cases?
    wx.config.pathset([log_dir]);

    // Keep running until Watchexec quits
    let _ = main.await.into_diagnostic()?;

    Ok(())
}
