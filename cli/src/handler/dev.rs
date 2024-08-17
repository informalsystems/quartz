use std::{process::exit, str::FromStr, time::Duration};

use async_trait::async_trait;
// todo get rid of this?
use miette::{IntoDiagnostic, Result};
use quartz_common::proto::core_client::CoreClient;
use tokio::{
    sync::{mpsc, watch},
    time::sleep,
};
use tracing::info;
use watchexec::Watchexec;
use watchexec_signals::Signal;

use crate::{
    cache,
    error::Error,
    handler::{utils::helpers::wasmaddr_to_id, Handler},
    request::{
        contract_build::ContractBuildRequest, contract_deploy::ContractDeployRequest,
        dev::DevRequest, enclave_build::EnclaveBuildRequest, enclave_start::EnclaveStartRequest,
        handshake::HandshakeRequest,
    },
    response::{dev::DevResponse, Response},
    Config,
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

        let (tx, rx) = mpsc::channel::<DevRebuild>(32);
        let _res = tx.send(DevRebuild::Init).await;

        if self.watch {
            tokio::spawn(watcher(tx));
        }

        let _res = dev_driver(rx, config.clone()).await;

        Ok(DevResponse.into())
    }
}

#[derive(Debug, Clone)]
enum DevRebuild {
    Init,
    Enclave,
    Contract,
}

async fn dev_driver(mut rx: mpsc::Receiver<DevRebuild>, config: Config) -> Result<(), Error> {
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

    // Config state can be held in memory here for contract addrs and etc
    while let Some(dev) = rx.recv().await {
        match dev {
            DevRebuild::Init => {
                info!("Launching quartz app...");

                // Build enclave
                let enclave_build = EnclaveBuildRequest {
                    release: false,
                    manifest_path: "../apps/transfers/enclave/Cargo.toml".parse().unwrap(),
                };
                let _eb_res = enclave_build
                    .handle(&config) // TODO: pass by ref
                    .await?;

                // Build contract
                let contract_build = ContractBuildRequest {
                    manifest_path: "../apps/transfers/contracts/Cargo.toml".parse().unwrap(),
                };
                let _cb_res = contract_build.handle(&config).await?;

                let new_shutdown_tx = spawn_enclave_start(&config).await?;

                let res = deploy_and_handshake(None, &config).await;

                // Set new contract or shutdown and return error
                match res {
                    Ok(res_contract) => {
                        // Set state
                        contract = res_contract;
                        shutdown_tx = Some(new_shutdown_tx);
                    }
                    Err(e) => {
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

                info!("Rebuilding Enclave...");
                if let Some(shutdown_tx) = shutdown_tx.clone() {
                    let _res = shutdown_tx.send(());
                }

                info!("Waiting 1 second for the enclave to shut down");
                sleep(Duration::from_secs(1)).await;

                let new_shutdown_tx = spawn_enclave_start(&config).await?;

                // todo: should not unconditionally deploy here
                let res = deploy_and_handshake(Some(&contract), &config).await;

                match res {
                    Ok(res_contract) => {
                        // Set state
                        contract = res_contract;
                        shutdown_tx = Some(new_shutdown_tx);
                    }
                    Err(e) => {
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

                info!("Rebuilding Contract...");

                if let Some(shutdown_tx) = shutdown_tx.clone() {
                    let res = deploy_and_handshake(None, &config).await;

                    match res {
                        Ok(res_contract) => contract = res_contract,
                        Err(e) => {
                            shutdown_tx
                                .send(())
                                .expect("Could not send signal on channel");

                            return Err(e);
                        }
                    }
                } else {
                    panic!("enclave should exist homie");
                }
            }
        }
    }

    Ok(())
}

// Spawns enclve start in a separate task which runs in the background
async fn spawn_enclave_start(config: &Config) -> Result<watch::Sender<()>, Error> {
    // In separate process, launch the enclave
    let (shutdown_tx, shutdown_rx) = watch::channel(());
    let enclave_start = EnclaveStartRequest {
        shutdown_rx: Some(shutdown_rx),
        use_latest_trusted: false, // TODO: collect arg for dev, maybe this goes in config?
    };

    let config_cpy = config.clone();

    tokio::spawn(async move {
        let res = enclave_start.handle(config_cpy).await?;

        Ok::<Response, Error>(res)
    });

    Ok(shutdown_tx)
}

// TODO: do not shutdown if cli calls fail, just print
async fn deploy_and_handshake(contract: Option<&str>, config: &Config) -> Result<String, Error> {
    info!("Waiting for enclave start to deploy contract and handshake");

    // Wait at most 30 seconds to connect to enclave
    let mut i = 30;
    while let Err(_) = CoreClient::connect(format!(
        "{}:{}",
        config.enclave_rpc_addr, config.enclave_rpc_port
    ))
    .await
    {
        sleep(Duration::from_secs(1)).await;
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
        contract.to_string()
    } else {
        // Deploy Contract request
        let contract_deploy = ContractDeployRequest {
            init_msg: serde_json::Value::from_str(r#"{"denom": "ucosm"}"#).expect("init msg didnt work"), // todo: receive from args
            label: "test".to_string(),
            wasm_bin_path: "../apps/transfers/contracts/target/wasm32-unknown-unknown/release/transfers_contract.wasm".into()
        };
        // Call handler
        let cd_res = contract_deploy.handle(config).await;

        // Return contract address or shutdown enclave & error
        let contract = if let Ok(Response::ContractDeploy(res)) = cd_res {
            res.contract_addr
        } else {
            return Err(Error::GenericErr(format!(
                "Deploy failed: {}",
                cd_res.expect_err("else")
            )));
        };

        contract
    };

    // Run handshake
    let handshake = HandshakeRequest {
        contract: wasmaddr_to_id(&contract).map_err(|_| Error::GenericErr(String::default()))?,
    };

    println!("handshake: {:?}", handshake);
    let h_res = handshake.handle(config).await;

    let h_res = if let Ok(Response::Handshake(res)) = h_res {
        res
    } else {
        return Err(Error::GenericErr(format!(
            "Handshake failed: {}",
            h_res.expect_err("else")
        )));
    };

    info!("Handshake complete\n{:?}", h_res);

    Ok(contract)
}

async fn watcher(tx: mpsc::Sender<DevRebuild>) -> Result<()> {
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
                println!("path:\n {:?}", path);
                if path.to_string_lossy().contains("-enclave") {
                    let _res = tx.send(DevRebuild::Enclave).await;
                } else if path.to_string_lossy().contains("-contract") {
                    let _res = tx.send(DevRebuild::Contract).await;
                }
            }

            action
        })
    })?;

    // Start the engine
    let main = wx.main();

    // Watch all files in quartz app directory
    wx.config.pathset([cache::log_dir().unwrap()]);

    // Keep running until Watchexec quits
    let _ = main.await.into_diagnostic()?;

    Ok(())
}
