# Getting Started

Here you'll get quickly up and running with an example Quartz application.

You can run this locally using a "mock" enclave (i.e. without real privacy or
attestations), or you can get access to a machine with Intel SGX enabled to run it securely.

TODO sections:

- Transfer Application - describe basic application
- Local Mock SGX - describe getting up and running locally with mock sgx (old demo instructions, updated with new quartz CLI) -done
- Real SGX - describe how to get setup quickly with SGX e.g. on Azure, and how to run the same example there - done needs review
- Public Testnet - describe how to deploy on Neutron testnet - TODO

## Transfer Application Template

The Transfer Application is a simple template / demo app designed to showcase very basic use of the Quartz framework. It allows users to deposit funds into a contract, transfer them privately within the contract's encrypted state, and ultimately withdraw whatever balance they have left or have accumulated. 

#### Key Features

1. **Deposit Funds**: Users can deposit funds into a smart contract.
2. **Private Transfers**: Users can transfer funds privately within the contract via encrypted transactions that are handled by Quartz (ie. processed by the enclave and remote attested to).
3. **Withdraw Funds**: Users can withdraw their funds from the contract based on their balance in the encrypted state.

#### Application Structure

The application is divided into:

1. **Frontend**: The user interface built with Next.js, cosmjs / graz.
2. **Contracts**: The backend application as a CosmWasm smart contract
3. **Enclave**: Code that executes off-chain and privately in an enclave

#### Setting Up the Application

To get started with the Transfer Application, follow these steps:

**Prerequisites**
Ensure you have the following installed:
Go: Required for building wasmd.
Make: Typically pre-installed on Linux systems.
Git: For cloning the repository.
Websocat: To listen to the events.


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

### Clone the Repository
First, clone the repository to your local machine.

```bash
    git clone ssh://git@github.com/informalsystems/cycles-quartz
    cd cycles-quartz
```



### Install a local daemon (local neutrond/wasmd)
Install a local daemon to be able to easily interact with a local or remote chain. 

a. Neutron 
```bash
	git clone -b main https://github.com/neutron-org/neutron.git
	cd neutron
	make install
```

After the installation, to verify everything is working run:

```bash
	neutrond version
```

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


### Run a local chain from docker

Follow the instructions on [the docker folder]()
a. Neutron 
```bash
	cd docker/neutron
	make start-docker-container
```

After the installation, you should see a `neutrond` container if you run

```
	docker ps
```

b. Wasmd 
```bash
	cd docker/wasmd
	make run
```

After the installation, you should see a `wasmd` container if you run

```
	docker ps
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

This command will build contract and enclave binaries, deploy, instantiate the contract, handshake and listen to the contract.

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
quartz init --name <new_app_name> --path /path/to/your/project
```

This command will create a new directory structure for your Quartz app:
```
new_app_name/
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

### Performing the Handshake + activating listener

To establish communication between the contract and the enclave, perform the handshake:

```bash
quartz --mock-sgx --app-dir "apps/transfers/" handshake --contract <CONTRACT_ADDRESS>
```

Replace `<CONTRACT_ADDRESS>` with the address you received when deploying the contract.

Make note of the handshake generate public key, as you'll need it for the `.env.local` files on the front-end.

```bash
2024-09-24T11:12:16.961641Z  INFO Handshake complete: 02360955ff74750f6ea0b539f41cce89451f591e4c835d0a5406e6effa96dd169d
```

Events coming from the contract will be logged following the handshake as they are retrieved by the listener.

```bash
2024-09-24T11:12:25.156779Z  INFO Enclave is listening for requests...
```

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
2. Use one of the provided accounts [wasmd_docker](docker/wasmd/accounts/) / [neutrond_docker](docker/neutrond/accounts/)  ) by importing it into your Keplr wallet or create a new one and fund it using the CLI with your `admin` / `val1` account.
3. Use the frontend to deposit funds into the contract.
4. Transfer funds privately between different accounts within the contract.
5. Withdraw funds from the contract back to your Keplr wallet.

All transactions will be processed automatically by the enclave, ensuring the privacy and confidentiality of your transfers.

This completes the basic setup and usage guide for the Transfer Application. In the next sections, we'll cover how to run this application with local Mock SGX, real SGX on Azure, and how to deploy it on the Neutron testnet.





## Local daemon and wallet 

For all **testing** purposes, you can use the wallets we provide in the `docker/wasmd/accounts` folder.

Once your `wasmd` is running, you can easily copy the wallets from docker to your local daemon `~/.config` folder:

Here's how you can copy the user wallets from the Docker instance to your local wasmd configuration:

1. First, ensure that your local wasmd instance is not running. If it is, stop it.

2. Locate the directory where the Docker instance stores the wallet data. Based on the provided `Dockerfile`, it appears to be `/root` inside the container.

3. Copy the wallet data from the Docker container to a temporary directory on your local machine:

```bash
docker cp -r wasmd:/root/.wasmd/keyring-test temp-keyring
```

This command copies the `keyring-test` directory from the Docker container to a `temp-keyring` directory in your current local directory.

4. Copy the wallet data from the temporary directory to your local wasmd configuration directory:

```bash
cp -r ~/.temp-keyring/* ~/.wasmd/keyring-test/
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

### Alternative method with docker running 
Now, let's add the admin key. We'll use the mnemonic from the admin.txt file in your Docker setup. First, let's view the contents of that file:

```bash
docker exec wasmd cat /tmp/accounts/admin.txt
```

You should see a mnemonic phrase (a series of words) at the end of this output.

Now, use this mnemonic to add the admin key to your local wasmd installation:
```bash
wasmd keys add admin --recover --keyring-backend=test
```

After completing these steps, your local wasmd instance should have the same `admin` wallets as the Docker instance. You can repeat the process for `alice`, `bob` and `charlie`. 

 You can now start your local wasmd instance, and it will use the copied wallets.

Note: Make sure to replace `wasmd` in the `docker cp` command with the actual name of your running Docker container if it differs.





## Working with an Azure Sgx 


Login via `ssh` into your Azure Sgx enabled machine. 

```bash
 ssh username@21.6.21.71
```

### Quickstart
Once logged in, install the `cli` with the following command:

```bash
    cargo install --path cli/
```

We now need to build the binaries.


### Build the Binaries
To build both the contract binaries, use the build command:

```bash
quartz --app-dir "apps/transfers/" contract build --contract-manifest "apps/transfers/contracts/Cargo.toml"
```


This command will compile the smart contract to WebAssembly and build the contract binary.


### Configuring and Running the Enclave

The following configuration assumes that the `wasmd` node will be running in the same Azure instance as the enclave. 
If you wish to use another enclave provider you have to make sure that `QUARTZ_NODE_URL` is set to the enclave address and port as an argument as in:

```
QUARTZ_NODE_URL=87.23.1.3:11090 && quartz --app-dir "apps/transfers/" contract deploy  --contract-manifest "apps/transfers/contracts/Cargo.toml"   --init-msg '{"denom":"ucosm"}'

```

If you wish to use another blockchain you have to make sure that `--node-url` is set to the chain address and port as an option in the `cli` as in:

```
QUARTZ_NODE_URL=127.0.0.1:11090 && quartz --app-dir "apps/transfers/" --node-url "https://92.43.1.4:26657" contract deploy  --contract-manifest "apps/transfers/contracts/Cargo.toml"   --init-msg '{"denom":"ucosm"}'

```



To configure and run the enclave, use the following commands:

```bash
  # Configure the enclave
quartz --app-dir "apps/transfers/" enclave  build
```


Before starting the enclave, you have to make sure that all relevant contracts (tcbinfo, dcap-verifier) have been instantiated as described below 
```bash
  # Start the enclave
QUARTZ_NODE_URL=127.0.0.1:11090 && quartz --app-dir "apps/transfers/" enclave start  --fmspc "00606A000000" --tcbinfo-contract "wasm1pk6xe9hr5wgvl5lcd6wp62236t5p600v9g7nfcsjkf6guvta2s5s7353wa" --dcap-verifier-contract "wasm107cq7x4qmm7mepkuxarcazas23037g4q9u72urzyqu7r4saq3l6srcykw2"
```

The enclave will start running and wait for commands.

### Deploying the Contract

With the enclave running, open a new terminal window to deploy the contract:

```bash
QUARTZ_NODE_URL=127.0.0.1:11090 && quartz --app-dir "apps/transfers/" contract deploy  --contract-manifest "apps/transfers/contracts/Cargo.toml"   --init-msg '{"denom":"ucosm"}'
```

Make note of the deployed contract address, as you'll need it for the next step.


```
2024-09-26T15:21:39.036639Z  INFO 🆔 Code ID: 66
2024-09-26T15:21:39.036640Z  INFO 📌 Contract Address: wasm1z0az3d9j9s3rjmaukxc58t8hdydu8v0d59wy9p6slce66mwnzjusy76vdq
{"ContractDeploy":{"code_id":66,"contract_addr":"wasm1z0az3d9j9s3rjmaukxc58t8hdydu8v0d59wy9p6slce66mwnzjusy76vdq"}}

```

### Performing the Handshake + activating listener

To establish communication between the contract and the enclave, perform the handshake:

```bash
quartz --app-dir "apps/transfers/" handshake --contract <CONTRACT_ADDRESS>
```

Replace `<CONTRACT_ADDRESS>` with the address you received when deploying the contract.

Make note of the handshake generate public key, as you'll need it for the `.env.local` files on the front-end.

```bash
2024-09-24T11:12:16.961641Z  INFO Handshake complete: 02360955ff74750f6ea0b539f41cce89451f591e4c835d0a5406e6effa96dd169d
```

Events coming from the contract will be logged following the handshake as they are retrieved by the listener.

```bash
2024-09-24T11:12:25.156779Z  INFO Enclave is listening for requests...
```


## Quartz cosmwasm packages 
### Get the FMSPC of the host machine
```
export QUOTE="/* quote generated during the handshake should work */"
cd utils/print-fmspc/
cargo run > /dev/null
```



### Deploying the `tcbinfo` contract
1. Build and store the contract on-chain
```
cargo run -- contract build --contract-manifest "../cosmwasm/packages/tcbinfo/Cargo.toml"
RES=$(wasmd tx wasm store ./target/wasm32-unknown-unknown/release/tcbinfo.wasm --from alice -y --output json --chain-id "testing" --gas-prices 0.0025ucosm --gas auto --gas-adjustment 1.3)
TX_HASH=$(echo $RES | jq -r '.["txhash"]')
```
2. Instantiate the contract using Intel's root CA cert.
```
CERT=$(sed ':a;N;$!ba;s/\n/\\n/g' ../cosmwasm/packages/quartz-tee-ra/data/root_ca.pem)
RES=$(wasmd query tx "$TX_HASH" --output json)
CODE_ID=$(echo $RES | jq -r '.logs[0].events[1].attributes[1].value')
wasmd tx wasm instantiate "$CODE_ID" "{\"root_cert\": \"$CERT\"}" --from "alice" --label "tcbinfo" --chain-id "testing" --gas-prices 0.0025ucosm --gas auto --gas-adjustment 1.3 -y --no-admin --output json	
TCB_CONTRACT=$(wasmd query wasm list-contract-by-code "$CODE_ID" --output json | jq -r '.contracts[0]')
```
3. Get the Tcbinfo for the given FMSPC.
```
HEADERS=$(wget -q -S -O - https://api.trustedservices.intel.com/sgx/certification/v4/tcb?fmspc=00606A000000 2>&1 >/dev/null)
TCB_INFO=$(wget -q -O - https://api.trustedservices.intel.com/sgx/certification/v4/tcb?fmspc=00606A000000)
export TCB_ISSUER_CERT=$(echo "$HEADERS" | 
        grep 'TCB-Info-Issuer-Chain:' | 
        sed 's/.*TCB-Info-Issuer-Chain: //' | 
        sed 's/%0A/\n/g' | 
        sed 's/%20/ /g' | 
        sed 's/-----BEGIN%20CERTIFICATE-----/-----BEGIN CERTIFICATE-----/' | 
        sed 's/-----END%20CERTIFICATE-----/-----END CERTIFICATE-----/' | 
        perl -MURI::Escape -ne 'print uri_unescape($_)' | 
        awk '/-----BEGIN CERTIFICATE-----/{flag=1; print; next} /-----END CERTIFICATE-----/{print; flag=0; exit} flag')

TCB_ISSUER_CERT=$(echo "$TCB_ISSUER_CERT" | sed ':a;N;$!ba;s/\n/\\n/g')
echo "TCB_INFO:"
echo "$TCB_INFO"
echo
echo "TCB_ISSUER_CERT:"
echo "$TCB_ISSUER_CERT"
```
4. Add the Tcbinfo for the given FMSPC to the contract (and test it with a query)
```
wasmd tx wasm execute "$TCB_CONTRACT" "{\"tcb_info\": $(echo "$TCB_INFO" | jq -Rs .), \"certificate\": \"$TCB_ISSUER_CERT\"}" --from admin --chain-id testing --gas auto --gas-adjustment 1.2 -y 
wasmd query wasm contract-state smart "$TCB_CONTRACT" '{"get_tcb_info": {"fmspc": "00606A000000"}}'
```

### Deploying the `quartz-dcap-verifier` contract
1. Build the contract
```
cargo run -- contract build --contract-manifest "../cosmwasm/packages/quartz-dcap-verifier/Cargo.toml"
```
2. Optimize the contract
In order to optimized the contract, you need to install `wasm-opt` v.119 see HOWTO below
```
wasm-opt -Oz ./target/wasm32-unknown-unknown/release/quartz_dcap_verifier.wasm -o ./target/wasm32-unknown-unknown/release/quartz_dcap_verifier.optimized.wasm
```
3. Store the optimized contract on-chain
```
RES=$(wasmd tx wasm store ./target/wasm32-unknown-unknown/release/quartz_dcap_verifier.optimized.wasm --from admin -y --output json --chain-id "testing" --gas-prices 0.0025ucosm --gas auto --gas-adjustment 1.3)
TX_HASH=$(echo $RES | jq -r '.["txhash"]')
RES=$(wasmd query tx "$TX_HASH" --output json)
CODE_ID=$(echo $RES | jq -r '.logs[0].events[1].attributes[1].value')
```
4. Instantiate the `quartz-dcap-verifier` contract.
```
wasmd tx wasm instantiate "$CODE_ID" null --from "admin" --label "dcap-verifier" --chain-id "testing" --gas-prices 0.0025ucosm --gas auto --gas-adjustment 1.3 -y --no-admin --output json
DCAP_CONTRACT=$(wasmd query wasm list-contract-by-code "$CODE_ID" --output json | jq -r '.contracts[0]')
```

### Quartz setup
```
quartz --app-dir "../apps/transfers/"
    --contract-manifest "../apps/transfers/contracts/Cargo.toml" \
    --unsafe-trust-latest \
    --init-msg '{"denom":"ucosm"}' \
     dev \
    --fmspc "00606A000000" \
    --tcbinfo-contract "$TCB_CONTRACT" \
    --dcap-verifier-contract "$DCAP_CONTRACT"
```




#### HOWTO Install `wasm-opt`

To install `wasm-opt` version 119 on an Azure SGX machine running Ubuntu, follow these steps:

### 1. **Update and install dependencies:**

Before installing `wasm-opt`, make sure your system is up-to-date and has the necessary build tools:

```bash
sudo apt update
sudo apt install -y build-essential cmake git
```

### 2. **Download and build `wasm-opt` version 119:**

The `wasm-opt` tool is part of the [Binaryen](https://github.com/WebAssembly/binaryen) project. To get version 119, you’ll need to clone the specific tag from the Binaryen GitHub repository.

```bash
git clone https://github.com/WebAssembly/binaryen.git
cd binaryen
git checkout version_119
```

### 3. **Build the project:**

Next, you'll build Binaryen and the `wasm-opt` tool:

```bash
cmake . && make
```

### 4. **Install `wasm-opt`:**

Once the build is complete, you can install `wasm-opt` globally on your system:

```bash
sudo make install
```

### 5. **Verify the installation:**

Finally, confirm that `wasm-opt` version 119 is installed correctly by running:

```bash
wasm-opt --version
```

This should return something like:

```
wasm-opt version_119
```

Now, `wasm-opt` version 119 should be properly installed on your Azure SGX machine running Ubuntu. 