use async_trait::async_trait;
use tokio::{sync::mpsc, try_join};
use tracing::{info, trace};

use crate::{
    error::Error,
    handler::Handler,
    request::{dev::DevRequest, enclave_build::EnclaveBuildRequest},
    response::{dev::DevResponse, Response},
    Config,
};

use std::{sync::Arc, thread::sleep, time::Duration};
// todo get rid of this?
use miette::{IntoDiagnostic, Result};
use watchexec::{
	command::{Command, Program, Shell, SpawnOptions},
	job::CommandState,
	Id, Watchexec,
};
use watchexec_events::{Event, Priority, ProcessEnd};
use watchexec_signals::Signal;

#[async_trait]
impl Handler for DevRequest {
    type Error = Error;
    type Response = Response;

    async fn handle(self, _config: Config) -> Result<Self::Response, Self::Error> {
        trace!("initializing directory structure...");

		if !self.watch {
			// Build enclave

			// In separate process, start enclave

			// Build contract

			// Deploy contract

			// Run handshake
	
			dispatch_dev(DevRebuild:: Both).await;
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

enum DevRebuild {
	Enclave,
	Contract,
	Both,
}

async fn dispatch_dev(dev: DevRebuild) {
	match dev {
		DevRebuild::Both => {
			info!("Launching quartz app...");

			let build = EnclaveBuildRequest { release: false, manifest_path: "../apps/transfers/enclave/Cargo.toml".parse().unwrap() };
			let res = build.handle(Config { mock_sgx: false }).await;		
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