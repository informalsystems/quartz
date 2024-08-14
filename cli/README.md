# quartz CLI

The `quartz` CLI tool is designed to streamline the development and deployment process of Quartz applications.

It provides helpful information about each command and its options. To get a list of all available subcommands and their
descriptions, use the `--help` flag:

```shell
$ quartz --help

Quartz 0.1.0
A CLI tool to manage Quartz applications

USAGE:
    quartz [SUBCOMMAND]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    init        Create base Quartz app directory from template
    build       Build the contract and enclave binaries
    enclave     Enclave subcommads to configure Gramine, build, sign, and start the enclave binary
    contract    Contract subcommads to build, deploy the WASM binary to the blockchain and call instantiate
    handshake   Run the handshake between the contract and enclave
```

## Installation

To install Quartz, ensure you have Rust and Cargo installed. Then run:

```shell
cargo install quartz
```

## Usage of subcommands

### Init

Initialize a new Quartz app directory structure with optional name and path arguments.

#### Usage

```shell
$ quartz init --help
quartz-init 
Create base Quartz app directory from template

USAGE:
    quartz init [OPTIONS]

OPTIONS:
    -n, --name <NAME>    Set the name of the Quartz app [default: <name of parent directory>]
    -p, --path <PATH>    Set the path where the Quartz app will be created [default: .]
    -h, --help           Print help information
```

#### Example

```shell
quartz init --name <app_name> --path <path>
```

This command will create the following directory structure at the specified path (or the current directory if no path is
provided):

```shell
$ tree /<path>/<app-name> -L 1
apps/transfers/
├── contracts/
├── enclave/
├── frontend/
└── README.md
```

## Intsructions to quickly setup the transfers app
You can use these instructions to run the transfers app. For your own app, you will need to adjust the env vars and paths as needed. 

> Note - Run all commands from within the `/cli` folder

```bash
# setup env vars in ALL THREE terminals
export MOCK_SGX=true
export NODE_URL=143.244.186.205:26657
export CHAIN_ID=testing

#-------------------------------------------------------------------------------
# TERMINAL 1 - setup enclave

# build enclave
cargo run -- enclave build --manifest-path "../apps/transfers/enclave/Cargo.toml"

# start enclave
cargo run -- enclave start --app-dir "../apps/transfers" --chain-id $CHAIN_ID

#-------------------------------------------------------------------------------
# TERMINAL 2 - After enclave is setup, setup contracts

# build contracts
cargo run -- --mock-sgx  contract build --manifest-path "../apps/transfers/contracts/Cargo.toml"

# deploy contracts
cargo run -- \
    --mock-sgx \
    contract deploy \
    --wasm-bin-path "../apps/transfers/contracts/target/wasm32-unknown-unknown/release/transfers_contract.wasm" \
    --init-msg '{"denom": "ucosm"}'

# retrieve contract addr from output and store in env
export CONTRACT=<YOUR_CONTRACT_ADDR>

# handshake
cargo run -- --mock-sgx handshake --app-dir "../apps/transfers/" --contract $CONTRACT

# listen - NOTE - still using bash instead of cli
bash ../apps/transfers/scripts/listen.sh $CONTRACT

#-------------------------------------------------------------------------------
# TERMINAL 3 - After contracts are setup, interact with them

export CONTRACT=<YOUR_CONTRACT_ADDR>

## example 1
cargo run -- contract tx --msg "\"deposit\"" --amount 1000ucosm --gas 200000 --contract $CONTRACT

## example 2
cargo run -- \
    contract tx \
    --msg "{\"query_request\": {\"emphemeral_pubkey\": \"$EPHEMERAL_PUBKEY\"}}" \
    --contract $CONTRACT
```
