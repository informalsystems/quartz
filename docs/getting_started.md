# Getting Started

Here you'll get quickly up and running with an example Quartz application.

You can run this locally using a "mock" enclave (i.e. without real privacy or
attestations), or you can get access to a machine with Intel SGX enabled to run it securely.

TODO sections:

- Transfer Application - describe basic application
- Local Mock SGX - describe getting up and running locally with mock sgx (old demo instructions, updated with new quartz CLI)
- Real SGX - describe how to get setup quickly with SGX e.g. on Azure, and how to run the same example there
- Public Testnet - describe how to deploy on Neutron testnet

## Transfer Application Template

The Transfer Application is a simple template / demo app designed to showcase how users can deposit funds into a contract, transfer them privately within the contract's encrypted state, and withdraw their funds. This application is built using various tools and technologies, including Rust, Go, and Next.js.

#### Key Features

1. **Deposit Funds**: Users can deposit funds into a smart contract.
2. **Private Transfers**: Users can transfer funds privately within the contract.
3. **Withdraw Funds**: Users can withdraw their funds from the contract.

#### Application Structure

The application is divided into several components, each responsible for different parts of the functionality. Here is a brief overview of the main components:

1. **Frontend**: The user interface built with Next.js, cosmjs / graz.
2. **Backend**: The server-side logic, including smart contracts written in Rust.
3. **Enclave**: A secure environment for executing sensitive operations.

#### Setting Up the Application

To get started with the Transfer Application, follow these steps:

**Prerequisites**
Ensure you have the following installed:
Go: Required for building wasmd.
Make: Typically pre-installed on Linux systems.
Git: For cloning the repository.
Websocat: To listen to the events.

### Clone the Repository
First, clone the repository to your local machine.

```bash
    git clone ssh://git@github.com/informalsystems/cycles-quartz
    cd cycles-quartz
```


### Install Dependencies
Install the necessary dependencies for both the frontend and backend.

```bash
    # Install Rust
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    rustup target add wasm32-unknown-unknown
	
    # Install Go tools
    export PATH="${PATH}:${HOME}/go/bin"
    source ~/.bashrc
    go install github.com/fullstorydev/grpcurl/cmd/grpcurl@latest
	 
	 # Install grpcurl
	 go install github.com/fullstorydev/grpcurl/cmd/grpcurl@latest	
 
	 # Install websocat
	 cargo install websocat
    
```

### Install a local daemon (local neutrond/wasmd)
Install a local daemon to be able to easily interact with a local or remote chain. 

a. Neutron 

b. Wasmd 

```bash
	git clone https://github.com/cosmwasm/wasmd/
	cd wasmd
	git checkout v0.44.0 / 0.52.0
	go install ./cmd/wasmd
```

After the installation, to verify everything is working run:

```bash
	wasmd version
```

### Installing the Quartz CLI
To install the Quartz CLI, run the following command from the `cycles-quartz` folder:

```bash
	cargo install --path cli/
```


### Quickstart on the basic using `dev`
To quickly get up and running, you can run the following `dev` command from the `cycles-quartz` folder:


```bash
quartz --mock-sgx --app-dir "apps/transfers/" dev   --unsafe-trust-latest  --contract-manifest "apps/transfers/contracts/Cargo.toml"   --init-msg '{"denom":"ucosm"}'
```

This command will build contract and enclave binaries, deploy, instantiate the contract, handshake and listen to the contract v

To stop the enclave:
```bash
 pkill -f quartz-app-tran 
```

To restart:
```bash
rm -rf apps/transfers/.cache && quartz --mock-sgx --app-dir "apps/transfers/" dev   --unsafe-trust-latest  --contract-manifest "apps/transfers/contracts/Cargo.toml"   --init-msg '{"denom":"ucosm"}'
```

From here you can jump to the section on the [front-end](#### Building the front-end Application).



### Create a new quartz app template

To create a new Quartz app, use the `init` command:

```bash
quartz init --name transfer_app --path /path/to/your/project
```

This command will create a new directory structure for your Quartz app:
```
transfer_app/
├── contracts/
├── enclave/
├── frontend/
└── README.md
```



### Build the Binaries
To build both the contract and enclave binaries, use the build command:

```bash
quartz --mock-sgx --app-dir "apps/transfers/" contract build --contract-manifest "apps/transfers/contracts/Cargo.toml"
```

This command will compile the smart contract to WebAssembly and build the enclave binary.

### Configuring and Running the Enclave

To configure and run the enclave, use the following commands:

```bash
  # Configure the enclave
quartz --mock-sgx --app-dir "apps/transfers/" enclave  build
```

```bash
  # Start the enclave
quartz --mock-sgx --app-dir "apps/transfers/" enclave  start
```

The enclave will start running and wait for commands.

### Deploying the Contract

With the enclave running, open a new terminal window to deploy the contract:

```bash
quartz --mock-sgx --app-dir "apps/transfers/" contract deploy  --contract-manifest "apps/transfers/contracts/Cargo.toml"   --init-msg '{"denom":"ucosm"}'

```

Make note of the deployed contract address, as you'll need it for the next step.

```bash
2024-09-24T11:11:51.233106Z  INFO 📌 Contract Address: wasm1suhgf5svhu4usrurvxzlgn54ksxmn8gljarjtxqnapv8kjnp4nrss5maay
```

### Performing the Handshake

To establish communication between the contract and the enclave, perform the handshake:

```bash
quartz --mock-sgx --app-dir "apps/transfers/" handshake --contract <CONTRACT_ADDRESS>
```

Replace `<CONTRACT_ADDRESS>` with the address you received when deploying the contract.

Make note of the handshake generate public key, as you'll need it for the `.env.local` files on the front-end.

```bash
2024-09-24T11:12:16.961641Z  INFO Handshake complete: 02360955ff74750f6ea0b539f41cce89451f591e4c835d0a5406e6effa96dd169d
```

### Running the Listener ???

To listen for events from the contract and trigger execution on the enclave:

```bash
quartz --mock-sgx --app-dir "apps/transfers/" listen --contract <CONTRACT_ADDRESS>
```

Again, replace `<CONTRACT_ADDRESS>` with your deployed contract address.


#### Building the front-end Application

Once the application is set up, you can interact with it through the frontend. Make sure you have the Keplr extension installed in your browser. You can then deposit, transfer, and withdraw funds using different Keplr accounts.


```bash
	# Go to the front-end folder
	cd frontend
	# Do a fresh install
	npm ci
    # Copy .env.example 
	cp .env.example .env.local
	# Edit .env.local by adding `<CONTRACT_ADDRESS>` and `<PUB_KEY>` 
	nano .env.local
```

##### Example of app required variables
```
NEXT_PUBLIC_TARGET_CHAIN=localWasm
NEXT_PUBLIC_ENCLAVE_PUBLIC_KEY=02360955ff74750f6ea0b539f41cce89451f591e4c835d0a5406e6effa96dd169d
NEXT_PUBLIC_TRANSFERS_CONTRACT_ADDRESS=wasm1jfgr0vgunezkhfmdy7krrupu6yjhx224nxtjptll2ylkkqhyzeshrspu9
```

Once the enviroment variables are set, the front-end application can be started with the following command:

```bash
	npm run dev
```


For more detailed instructions on setting up and using the application, refer to the [README](apps/transfers/README.md) file.



## Interacting with the Application

Once you have the frontend running and connected to your local blockchain (or testnet), you can interact with the Transfer Application:

1. Ensure you have the Keplr wallet extension installed in your browser.
2. Use one of the provided accounts [wasmd_docker](docker/wasmd/accounts/) ) by importing it into your Keplr wallet or create a new one and fund it using the CLI with your `admin` / `val1` account.
3. Use the frontend to deposit funds into the contract.
4. Transfer funds privately between different accounts within the contract.
5. Withdraw funds from the contract back to your Keplr wallet.

All transactions will be processed automatically by the enclave, ensuring the privacy and confidentiality of your transfers.

This completes the basic setup and usage guide for the Transfer Application. In the next sections, we'll cover how to run this application with local Mock SGX, real SGX on Azure, and how to deploy it on the Neutron testnet.


## Local daemon and wallet 

For all **testing** purposes, you can yse the wallets we provide in the `docker/wasmd/accounts` folder.

Once your `wasmd` is running, you can easily copy the wallets from docker to your local daemon `~/.config` folder:

Here's how you can copy the user wallets from the Docker instance to your local wasmd configuration:

1. First, ensure that your local wasmd instance is not running. If it is, stop it.

2. Locate the directory where the Docker instance stores the wallet data. Based on the provided `Dockerfile`, it appears to be `/root` inside the container.

3. Copy the wallet data from the Docker container to a temporary directory on your local machine:

```bash
docker cp wasmd:/root/.wasmd/keyring-test ./temp-keyring
```

This command copies the `keyring-test` directory from the Docker container to a `temp-keyring` directory in your current local directory.

4. Copy the wallet data from the temporary directory to your local wasmd configuration directory:

```bash
cp -r ./temp-keyring/* ~/.wasmd/keyring-test/
```

This command copies the contents of the `temp-keyring` directory to your local `~/.wasmd/keyring-test/` directory.

5. Clean up the temporary directory:

```bash
rm -rf ./temp-keyring
```

6. Verify that the wallets have been copied successfully by running the following command:

```bash
wasmd keys list --keyring-backend=test
```

This should display the list of wallets that were present in the Docker container.

After completing these steps, your local wasmd instance should have the same wallets as the Docker instance. You can now start your local wasmd instance, and it will use the copied wallets.

Note: Make sure to replace `wasmd` in the `docker cp` command with the actual name of your running Docker container if it differs.




































