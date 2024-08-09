use async_trait::async_trait;
use tokio::{sync::{mpsc, oneshot}, time::sleep, try_join};
use tracing::{info, trace};

use crate::{
    error::Error,
    handler::Handler,
    request::{contract_build::ContractBuildRequest, contract_deploy::ContractDeployRequest, dev::DevRequest, enclave_build::EnclaveBuildRequest, enclave_start::EnclaveStartRequest, handshake::HandshakeRequest},
    response::{dev::DevResponse, Response},
    Config,
};

use std::{env, str::FromStr, sync::Arc, time::Duration};
use crate::handler::utils::helpers::wasmaddr_to_id;

// todo get rid of this?
use miette::{IntoDiagnostic, Result};
use watchexec::{
	command::{Command, Program, Shell},
	job::CommandState,
	Id, Watchexec,
};
use watchexec_events::{Event, Priority, ProcessEnd};
use watchexec_signals::Signal;

#[async_trait]
impl Handler for DevRequest {
    type Error = Error;
    type Response = Response;

    async fn handle(self, config: Config) -> Result<Self::Response, Self::Error> {
        trace!("initializing directory structure...");

		if !self.watch {
			// Build enclave
			let enclave_build = EnclaveBuildRequest { release: false, manifest_path: "../apps/transfers/enclave/Cargo.toml".parse().unwrap() };
			let _eb_res = enclave_build.handle(Config { mock_sgx: config.mock_sgx }).await?;		

			// Build contract
			let contract_build = ContractBuildRequest { manifest_path: "../apps/transfers/contracts/Cargo.toml".parse().unwrap() };
			let _cb_res = contract_build.handle(Config { mock_sgx: config.mock_sgx }).await?;

			// In separate process, launch the enclave
			let (start_tx, start_rx) = oneshot::channel();
			let enclave_start = EnclaveStartRequest { app_dir: self.app_dir.clone(), chain_id: "testing".to_string(), ready_signal: Some(start_tx) };
			
			let enclave_start_handle: tokio::task::JoinHandle<Result<Response, Error>> = tokio::spawn(async move {
					let res: Response = enclave_start.handle(Config { mock_sgx: config.mock_sgx }).await?;
					
					Ok(res)
			});

			info!("Waiting for enclave start to deploy contract and handshake");
			sleep(Duration::from_secs(3)).await;
			if let Ok(_) = start_rx.await {
				// Calls which interact with enclave
				info!("Enclave start message received");

				// Deploy contract
				let contract_deploy = ContractDeployRequest {
					init_msg: serde_json::Value::from_str("{}").map_err(|e| Error::GenericErr(e.to_string()))?,
					node_url: default_node_url(),
					chain_id: "testing".parse().map_err(|_| Error::GenericErr(String::default()))?,
					sender: "admin".to_string(),
					label: "llavel".to_string(),
					wasm_bin_path: "../apps/mtcs/contracts/cw-tee-mtcs/target/wasm32-unknown-unknown/release/cw_tee_mtcs.wasm".parse().map_err(|_| Error::GenericErr(String::default()))?,
				};

				let cd_res = contract_deploy.handle(Config { mock_sgx: config.mock_sgx }).await?;

				let contract = if let Response::ContractDeploy(res) = cd_res {
					res.contract_addr
				} else {
					return Err(Error::GenericErr("Deploy failed".to_string()));
				};

				// Run handshake
				let handshake = HandshakeRequest {
					contract: wasmaddr_to_id(&contract).map_err(|_| Error::GenericErr(String::default()))?,
					port: 11090u16,
					sender: "admin".to_string(),
					chain_id: "testing".parse().map_err(|_| Error::GenericErr(String::default()))?,
					node_url: default_node_url(),
					enclave_rpc_addr: default_rpc_addr(),
					app_dir: self.app_dir.clone(),
				};

				let h_res = handshake.handle(Config { mock_sgx: config.mock_sgx }).await?;
				
				info!("Handshake complete\n{:?}", h_res);

				info!("Enclave listening...");
				return enclave_start_handle.await.map_err(|_| Error::GenericErr(String::default()))?;
			} else {
				println!("did not get send signal");
			}
			// dispatch_dev(DevRebuild:: Both).await;
		} else {
			// Run listening process 

			let (tx, rx) = mpsc::channel::<DevRebuild>(32);
			let watcher_handler = tokio::spawn(watcher(self, tx));
			let message_handler = tokio::spawn(dev_driver(rx));
		
			// Use try_join! to wait for both tasks to complete.
			let (_, _) = try_join!(watcher_handler, message_handler).map_err(|e| Error::GenericErr(e.to_string()))?;
		}

			
        Ok(DevResponse.into())
    }
}


fn default_rpc_addr() -> String {
    env::var("RPC_URL").unwrap_or_else(|_| "http://127.0.0.1".to_string())
}

fn default_node_url() -> String {
    env::var("NODE_URL").unwrap_or_else(|_| "http://127.0.0.1:26657".to_string())
}


enum DevRebuild {
	Enclave,
	Contract,
	Both,
}

async fn dispatch_dev(dev: DevRebuild) {
	match dev {
		DevRebuild::Both => {
			info!("Launching quartz app...");

			let enclave_build = EnclaveBuildRequest { release: false, manifest_path: "../apps/transfers/enclave/Cargo.toml".parse().unwrap() };
			let res = enclave_build.handle(Config { mock_sgx: false }).await;		

		}
		DevRebuild::Enclave => {
			info!("Rebuilding Enclave...");
			let build = EnclaveBuildRequest { release: false, manifest_path: "../apps/transfers/enclave/Cargo.toml".parse().unwrap() };
			let res = build.handle(Config { mock_sgx: false }).await;		
		},
		DevRebuild::Contract => todo!(),
	}
} 

async fn dev_driver(mut rx: mpsc::Receiver<DevRebuild>) {
	// Config state can be held in memory here for contract addrs and etc
    while let Some(dev) = rx.recv().await {
		dispatch_dev(dev).await;
    }
}

async fn watcher(args: DevRequest, tx: mpsc::Sender<DevRebuild>) -> Result<()> {
	let build_id = Id::default();
	let DevRequest {
		app_dir,
		..
	} = args;

	let wx = Watchexec::new_async(move |mut action| {
		let tx = tx.clone();
		
		Box::new(async move {
			if action.signals().any(|sig| sig == Signal::Interrupt) {
				eprintln!("[Quitting...]");
				action.quit();
				return action;
			}

			// Defining the function that gets ran upon a detected code change
			// We run cargo check to only update when the codebase compiles
			let check = action.get_or_create_job(build_id, || {
				Arc::new(Command {
					program: Program::Shell {
						shell: Shell::new("bash"),
						command: "
							cargo check --package 'quartz-app-transfers-enclave
						".into(),
						args: Vec::new(),
					},
					options: Default::default(),
        	    }
			)});

			// Look for start event to launch quartz app
			if action.events.iter().any(|event| event.tags.is_empty())  {
				tx.send(DevRebuild::Both).await.unwrap();
				
				return action;
			}

			// Look for updates to filepaths
			if action.paths().next().is_some() {
				check.restart().await;
			}

			check.to_wait().await;
			// If job (cargo check) succeeds, then message dispatcher to run quartz-cli logic
			check
				.run(move |context| {
					if let CommandState::Finished {
						status: ProcessEnd::Success,
						..
					} = context.current	
					{
                        tokio::spawn(async move {
							tx.send(DevRebuild::Enclave).await.unwrap();
                        });
					}
				})
				.await;	

			action
		})
	})?;

	// Start the engine
	let main = wx.main();

	// Send an event to start
	wx.send_event(Event::default(), Priority::Urgent)
		.await
		.unwrap();

	// Watch all files in quartz app directory
	wx.config
		.throttle(Duration::new(5, 0))
		.pathset([app_dir]);

	// Keep running until Watchexec quits
	let _ = main.await.into_diagnostic()?;

	Ok(())
}