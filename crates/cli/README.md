# quartz CLI

A CLI tool to manage Quartz applications. The `quartz` CLI tool is designed to streamline the development and deployment
process of Quartz applications.

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
cargo install quartz-rs
```

## Usage

See the [getting started](/docs/getting_started.md).

Run `quartz init` to copy the example app into a new directory. Quartz apps are
organized like:

```shell
myapp/
├── contracts/
├── enclave/
├── frontend/
└── README.md
```
