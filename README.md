# cycles-quartz

A Rust implementation of the cycles protocol and the Quartz app framework.

This repository contains the following components -

### Apps

Quartz applications, each consisting of CosmWasm smart contracts, Gramine based sidecar enclaves and accompanying ZK
proofs.

Currently implemented apps -

* [MTCS](apps/mtcs) - The default app which implements Multilateral Trade Credit Set-off.

### Core

The Quartz core implementation including -

* Core handlers and types for Quartz
* Intel SGX remote attestation (RA) primitives
* Light client and merkle proofs for CosmWasm storage

### CosmWasm packages

CosmWasm packages for the core Quartz framework and remote attestation verification.

### Utils

Utilities for supporting Quartz development and  -

* [cw-prover](utils/cw-prover) - Retrieve a merkle-proof for CosmWasm state
* [cycles-sync](utils/cycles-sync) - Sync obligations and setoffs
  with [Obligato](https://github.com/informalsystems/obligato)
* [tm-prover](utils/tm-prover) - Generate light client and merkle proofs for CosmWasm storage in a format that Quartz
  understands

## Contributing

If you're interested in contributing, please comment on a relevant issue (if there is one) or open a new one!
See [CONTRIBUTING.md](CONTRIBUTING.md).

## Resources

* [Cycles website](https://cycles.money/)
* [Cycles Spec](docs/spec)
* [Quartz protobuf definitions](core/quartz-proto)

## License

TBD
