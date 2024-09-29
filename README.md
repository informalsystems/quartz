# Quartz

Quartz is a flexible framework for privacy-preserving computation via Trusted Execution
Environments (TEEs) organized and secured by smart contracts.

_**Why?**_ Development of Quartz was motivated by the privacy needs of the [Cycles Protocol][cycles],
which adopts a TEE-based ZK execution sidecar for private smart contracts. 
For background on how to think about different privacy preserving
technologies (FHE vs MPC vs ZKP vs TEE), see [How to win friends and TEE-fluence
people][how_to_win_friends_talk] and the associated [tweet
thread][how_to_win_friends_thread].

_**What?**_ With Quartz, data in smart contracts can be encrypted, while computation happens
privately off-chain via TEEs like SGX. Each contract can control what code runs in the
enclave, when it runs, and who is permitted to run it.

_**How?**_ At the heart of Quartz is a light-client protocol handshake between the enclave and the
smart contract which gives the smart contract control over when, how, and by who
the enclave code is run. This significantly reduces the surface area of TEEs.
See [How it Works][how_it_works].

_**Where?**_ Quartz currently targets the CosmWasm smart contract environment and the Intel SGX enclave. 
Other environments and TEEs remain for future work.

_**Who?**_ Quartz is (currently) for any CosmWasm developer interested in adding privacy or secure off-chain compute to their contracts and applications.

_**When?**_ Early, non-production versions of Quartz are available now for building
example applications. Production features and requirements are in development.
See [Future Work][future_work]

## Docs

- [Getting Started][getting_started] - Get a simple example app up and running in 5 minutes
- [How it Works][how_it_works] - How smart contracts and enclaves communicate securely
- [TEE Security][tees] - Resources on TEE security 
- [Building Applications][building_apps] - How to build Quartz applications
- [Future Work][future_work] - Roadmap and future work for security, flexibility, and
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


## License

TBD

[cycles]: https://cycles.money
[getting_started]: /docs/getting_started.md
[how_it_works]: /docs/how_it_works.md
[building_apps]: /docs/building_apps.md
[tees]: /docs/tees.md
[future_work]: /docs/roadmap.md
[how_to_win_friends_talk]: https://www.youtube.com/watch?v=XwKIt5XYyqw
[how_to_win_friends_thread]: https://x.com/buchmanster/status/1816084691784720887
