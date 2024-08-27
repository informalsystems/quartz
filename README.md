# Quartz

Quartz is a flexible framework for privacy-preserving computation via Trusted Execution
Environments (TEEs) organized and secured by smart contracts.

With Quartz, data in smart contracts can be encrypted, while computation happens
privately off-chain via enclaves. Each contract can control what code runs in the
enclave, when it runs, and who is permitted to run it.

Quartz currently targets the CosmWasm smart contract environment and the Intel SGX enclave. 
Other environments and TEEs remain for future work.

At the heart of quartz is a light-client protocol handshake between the enclave and the
smart contract which gives the smart contract control over when, how, and by who
the enclave code is run. This significantly reduces the surface area of TEEs.

NOTE: Quartz still requires secure protocol design from application developers.
Quartz is a low-level framework that provides secure compute environments
for private compute, but is not a complete  out-of-the box solution - application developers
must still define a secure data privacy model. Quartz DOES NOT specify how application data is padded, structured,
 and stored on chain in order to not leak information. For now this remains the
responsibility of the developer. While this provides maximum freedom and
control to develop applications, with great power comes great responsibility.

For how to think about different privacy preserving
technologies (FHE vs MPC vs ZKP vs TEE), see [How to win friends and TEE-fluence
people](TODO) and the associated [tweet thread](TODO).

## Docs

- [Getting Started](./docs/getting_started.md) - Get a simple example app up and running in 5 minutes
- [How it Works](./docs/how_it_works.md) - How smart contracts and enclaves communicate securely
- [TEE Security](./docs/tees.md) - Resources on TEE security 
- [Building Applications](./docs/building_apps.md) - How to build Quartz applications
- [Future Work](./docs/roadmap.md) - Roadmap and future work for security, flexibility, and
  more

## This Repo

This repository contains the following components -

### Apps

Example Quartz applications, including CosmWasm smart contracts, Gramine based sidecar enclave, and demo front end

Currently implemented apps -

* [Transfer](apps/transfer) - The default transfer app which allows private transfer of assets

### Core

The Quartz core implementation including -

* light-client-proofs: Light client and merkle proofs for CosmWasm storage
* quartz-proto: protobuf message types for quartz handshake between enclave and
  contract
* quartz: Intel SGX remote attestation (RA) primitives and quartz handshake logic 

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
