# Quartz: Getting Started Guide

## Table of Contents
- [Introduction](#introduction)
- [Quick Start](#quick-start)
- [Transfer Application Template](#transfer-application-template)
- [Setting Up the Application](#setting-up-the-application)
  - [Prerequisites](#prerequisites)
  - [Installation Steps](#installation-steps)
- [Building and Deploying](#building-and-deploying)
- [Running the Application](#running-the-application)
- [Working with Azure SGX](#working-with-azure-sgx)
- [Quartz CosmWasm Packages](#quartz-cosmwasm-packages)
- [Troubleshooting and FAQ](#troubleshooting-and-faq)
- [Glossary](#glossary)

## Introduction

This guide will help you get up and running with an example Quartz application. You can run this locally using a "mock" enclave (without real privacy or attestations), or you can use a machine with Intel SGX enabled for secure execution.

> **Note**: This guide assumes familiarity with blockchain concepts and basic smart contract development.

## Quick Start

For those who want to get started quickly:

1. Install dependencies (Rust, Go, Git, Websocat)
2. Clone the repository: `git clone ssh://git@github.com/informalsystems/cycles-quartz`
3. Install Quartz CLI: `cargo install --path cli/`
4. Run the development environment:
   ```bash
   quartz --mock-sgx --app-dir "apps/transfers/" dev \
   --unsafe-trust-latest \
   --contract-manifest "apps/transfers/contracts/Cargo.toml" \
   --init-msg '{"denom":"ucosm"}'
   ```
5. Set up the frontend (see [Building the front-end Application](#building-the-front-end-application))

For more detailed instructions, continue reading the following sections.

## Transfer Application Template

The Transfer Application is a simple template designed to showcase how users can deposit funds into a contract, transfer them privately within the contract's encrypted state, and withdraw their funds.

### Key Features
- Deposit funds into a smart contract
- Transfer funds privately within the contract
- Withdraw funds from the contract

### Application Structure
1. **Frontend**: User interface built with Next.js, cosmjs / graz
2. **Backend**: Server-side logic, including smart contracts written in Rust
3. **Enclave**: Secure environment for executing sensitive operations

## Setting Up the Application

### Prerequisites

Ensure you have the following installed:
- Go: Required for building wasmd
- Make: Typically pre-installed on Linux systems
- Git: For cloning the repository
- Websocat: To listen to events

### Installation Steps

1. Install Rust:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup target add wasm32-unknown-unknown
   ```

2. Install Go tools:
   ```bash
   export PATH="${PATH}:${HOME}/go/bin"
   source ~/.bashrc
   go install github.com/fullstorydev/grpcurl/cmd/grpcurl@latest
   ```

3. Install websocat:
   ```bash
   cargo install websocat
   ```

4. Clone the repository:
   ```bash
   git clone ssh://git@github.com/informalsystems/cycles-quartz
   cd cycles-quartz
   ```

5. Install a local daemon (choose one):
   
   a. Neutron:
   ```bash
   git clone -b main https://github.com/neutron-org/neutron.git
   cd neutron
   make install
   ```
   
   b. Wasmd:
   ```bash
   git clone https://github.com/cosmwasm/wasmd/
   cd wasmd
   git checkout v0.44.0
   go install ./cmd/wasmd
   ```

6. Run a local chain from docker (choose one):
   
   a. Neutron:
   ```bash
   cd docker/neutron
   make start-docker-container
   ```
   
   b. Wasmd:
   ```bash
   cd docker/wasmd
   make run
   ```

7. Install the Quartz CLI:
   ```bash
   cargo install --path cli/
   ```

## Building and Deploying

1. Build the binaries:
   ```bash
   quartz --mock-sgx --app-dir "apps/transfers/" contract build --contract-manifest "apps/transfers/contracts/Cargo.toml"
   ```

2. Configure and run the enclave:
   ```bash
   quartz --mock-sgx --app-dir "apps/transfers/" enclave build
   quartz --mock-sgx --app-dir "apps/transfers/" enclave start
   ```

3. Deploy the contract:
   ```bash
   quartz --mock-sgx --app-dir "apps/transfers/" contract deploy \
   --contract-manifest "apps/transfers/contracts/Cargo.toml" \
   --init-msg '{"denom":"ucosm"}'
   ```

4. Perform the handshake:
   ```bash
   quartz --mock-sgx --app-dir "apps/transfers/" handshake --contract <CONTRACT_ADDRESS>
   ```

> **Important**: Make note of the contract address and public key generated during deployment and handshake.

## Running the Application

### Building the front-end Application

1. Navigate to the frontend folder:
   ```bash
   cd frontend
   ```

2. Install dependencies:
   ```bash
   npm ci
   ```

3. Set up environment variables:
   ```bash
   cp .env.example .env.local
   nano .env.local
   ```

4. Start the frontend:
   ```bash
   npm run dev
   ```

### Interacting with the Application

1. Ensure you have the Keplr wallet extension installed in your browser.
2. Import a test account into Keplr or create a new one and fund it.
3. Use the frontend to deposit funds into the contract.
4. Transfer funds privately between different accounts within the contract.
5. Withdraw funds from the contract back to your Keplr wallet.


## Working with Azure SGX

Login via `ssh` into your Azure SGX enabled machine:

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

```bash
QUARTZ_NODE_URL=87.23.1.3:11090 && quartz --app-dir "apps/transfers/" contract deploy  --contract-manifest "apps/transfers/contracts/Cargo.toml"   --init-msg '{"denom":"ucosm"}'
```

If you wish to use another blockchain you have to make sure that `--node-url` is set to the chain address and port as an option in the `cli` as in:

```bash
QUARTZ_NODE_URL=127.0.0.1:11090 && quartz --app-dir "apps/transfers/" --node-url "https://92.43.1.4:26657" contract deploy  --contract-manifest "apps/transfers/contracts/Cargo.toml"   --init-msg '{"denom":"ucosm"}'
```

To configure and run the enclave, use the following commands:

```bash
# Configure the enclave
quartz --app-dir "apps/transfers/" enclave build
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

Make note of the deployed contract address, as you'll need it for the next step. You should see output similar to:

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

Make note of the handshake generate public key, as you'll need it for the `.env.local` files on the front-end. You should see output similar to:

```bash
2024-09-24T11:12:16.961641Z  INFO Handshake complete: 02360955ff74750f6ea0b539f41cce89451f591e4c835d0a5406e6effa96dd169d
```

Events coming from the contract will be logged following the handshake as they are retrieved by the listener:

```bash
2024-09-24T11:12:25.156779Z  INFO Enclave is listening for requests...
```

## Quartz CosmWasm Packages

### Get the FMSPC of the host machine

```bash
export QUOTE="/* quote generated during the handshake should work */"
cd utils/print-fmspc/
cargo run > /dev/null
```

### Deploying the `tcbinfo` contract

1. Build and store the contract on-chain
```bash
cargo run -- contract build --contract-manifest "../cosmwasm/packages/tcbinfo/Cargo.toml"
RES=$(wasmd tx wasm store ./target/wasm32-unknown-unknown/release/tcbinfo.wasm --from alice -y --output json --chain-id "testing" --gas-prices 0.0025ucosm --gas auto --gas-adjustment 1.3)
TX_HASH=$(echo $RES | jq -r '.["txhash"]')
```

2. Instantiate the contract using Intel's root CA cert.
```bash
CERT=$(sed ':a;N;$!ba;s/\n/\\n/g' ../cosmwasm/packages/quartz-tee-ra/data/root_ca.pem)
RES=$(wasmd query tx "$TX_HASH" --output json)
CODE_ID=$(echo $RES | jq -r '.logs[0].events[1].attributes[1].value')
wasmd tx wasm instantiate "$CODE_ID" "{\"root_cert\": \"$CERT\"}" --from "alice" --label "tcbinfo" --chain-id "testing" --gas-prices 0.0025ucosm --gas auto --gas-adjustment 1.3 -y --no-admin --output json	
TCB_CONTRACT=$(wasmd query wasm list-contract-by-code "$CODE_ID" --output json | jq -r '.contracts[0]')
```

3. Get the Tcbinfo for the given FMSPC.
```bash
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
```bash
wasmd tx wasm execute "$TCB_CONTRACT" "{\"tcb_info\": $(echo "$TCB_INFO" | jq -Rs .), \"certificate\": \"$TCB_ISSUER_CERT\"}" --from admin --chain-id testing --gas auto --gas-adjustment 1.2 -y 
wasmd query wasm contract-state smart "$TCB_CONTRACT" '{"get_tcb_info": {"fmspc": "00606A000000"}}'
```

### Deploying the `quartz-dcap-verifier` contract

1. Build the contract
```bash
cargo run -- contract build --contract-manifest "../cosmwasm/packages/quartz-dcap-verifier/Cargo.toml"
```

2. Optimize the contract
In order to optimize the contract, you need to install `wasm-opt` v.119. See the HOWTO section below for installation instructions.
```bash
wasm-opt -Oz ./target/wasm32-unknown-unknown/release/quartz_dcap_verifier.wasm -o ./target/wasm32-unknown-unknown/release/quartz_dcap_verifier.optimized.wasm
```

3. Store the optimized contract on-chain
```bash
RES=$(wasmd tx wasm store ./target/wasm32-unknown-unknown/release/quartz_dcap_verifier.optimized.wasm --from admin -y --output json --chain-id "testing" --gas-prices 0.0025ucosm --gas auto --gas-adjustment 1.3)
TX_HASH=$(echo $RES | jq -r '.["txhash"]')
RES=$(wasmd query tx "$TX_HASH" --output json)
CODE_ID=$(echo $RES | jq -r '.logs[0].events[1].attributes[1].value')
```

4. Instantiate the `quartz-dcap-verifier` contract.
```bash
wasmd tx wasm instantiate "$CODE_ID" null --from "admin" --label "dcap-verifier" --chain-id "testing" --gas-prices 0.0025ucosm --gas auto --gas-adjustment 1.3 -y --no-admin --output json
DCAP_CONTRACT=$(wasmd query wasm list-contract-by-code "$CODE_ID" --output json | jq -r '.contracts[0]')
```

### Quartz setup
```bash
quartz --app-dir "../apps/transfers/" \
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

1. **Update and install dependencies:**

```bash
sudo apt update
sudo apt install -y build-essential cmake git
```

2. **Download and build `wasm-opt` version 119:**

```bash
git clone https://github.com/WebAssembly/binaryen.git
cd binaryen
git checkout version_119
```

3. **Build the project:**

```bash
cmake . && make
```

4. **Install `wasm-opt`:**

```bash
sudo make install
```

5. **Verify the installation:**

```bash
wasm-opt --version
```

This should return something like:

```
wasm-opt version_119
```


## Troubleshooting and FAQ

1. **Q: The enclave fails to start. What should I do?**
   A: Ensure all dependencies are correctly installed and that you're using the correct version of each tool.

2. **Q: I'm getting a "contract not found" error during handshake. How do I fix this?**
   A: Double-check that you're using the correct contract address from the deployment step.

3. **Q: The frontend isn't connecting to the blockchain. What's wrong?**
   A: Verify that your `.env.local` file has the correct contract address and public key.

For more issues, please refer to our GitHub issues page or community forums.

## Glossary

- **Enclave**: A protected area of execution in memory.
- **SGX (Software Guard Extensions)**: Intel's technology for hardware-based isolation and memory encryption.
- **FMSPC**: Flexible Memory Sharing Protocol Component.
- **TCB**: Trusted Computing Base.
- **DCAP**: Data Center Attestation Primitives.
- **Wasmd**: Go implementation of a Cosmos SDK-based blockchain with WebAssembly smart contracts.
- **Neutron**: A CosmWasm-enabled blockchain built with the Cosmos SDK.