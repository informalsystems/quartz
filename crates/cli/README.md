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
