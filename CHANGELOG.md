# Changelog

## Release: v0.2.0

This release features a complete redesign of the enclave API (AKA Host-enclave separation) that -

- clearly separates the trusted/untrusted components of the app enclave code
- extracts more reusable code into the core enclave
- provides cleaner and more expressive abstractions

This means app devs now write up to ~20% less code.

The release also includes a new example app (called pingpong), numerous bug-fixes, API improvements, and better crate
documentation.

**Note:** this release contains multiple breaking changes.

### Features

- feat(enclave): API improvements to Store and KeyManager ([#299](https://github.com/informalsystems/quartz/pull/299))
- feat(enclave): allow app devs to define the pk type ([#297](https://github.com/informalsystems/quartz/pull/297))
- feat(enclave): add sequence number for replay protection ([#252](https://github.com/informalsystems/quartz/pull/252))
- feat(cw-client): add pay amount field to tx_execute ([#275](https://github.com/informalsystems/quartz/pull/275))
- feat(contract): Impl #[derive(UserData)] and improve
  naming ([#284](https://github.com/informalsystems/quartz/pull/284))
- feat(enclave): Host-enclave separation & redesign ([#283](https://github.com/informalsystems/quartz/pull/283))
- feat(examples): new template app ([#271](https://github.com/informalsystems/quartz/pull/271))

### Bug fixes

- fix(contract): UserData derive macro to avoid having users reimport
  stuff ([#303](https://github.com/informalsystems/quartz/pull/303))
- fix: add check for matching proof key ([#251](https://github.com/informalsystems/quartz/pull/251))
- fix(enclave): core include paths ([#257](https://github.com/informalsystems/quartz/pull/257))
- fix(enclave): proto build ([#256](https://github.com/informalsystems/quartz/pull/256))
- fix(cli): Update paths to public repo ([#258](https://github.com/informalsystems/quartz/pull/258))

### Refactor

- refactor: Remove all epoch related code ([#285](https://github.com/informalsystems/quartz/pull/285))
- refactor(enclave): remove core build.rs and copy data
  files ([#259](https://github.com/informalsystems/quartz/pull/259))

### Docs

- docs: Add comprehensive doc comments for core enclave traits, fns and
  types ([#302](https://github.com/informalsystems/quartz/pull/302))
- docs fixes ([#260](https://github.com/informalsystems/quartz/pull/260))
- Update docs ([#262](https://github.com/informalsystems/quartz/pull/262))
- Fix: Update on getting_started / tcbinfo ([#278](https://github.com/informalsystems/quartz/pull/278))
- fix(docs): checkout release version in getting_started.md ([#276](https://github.com/informalsystems/quartz/pull/276))
- fix(docs): getting started for docker and neutrond ([#264](https://github.com/informalsystems/quartz/pull/264))

### Build & CI

- build: add unsafe-trust-latest and contract-manifest
  defaults ([#292](https://github.com/informalsystems/quartz/pull/292))
- Add block pruning to neutrond docker ([#288](https://github.com/informalsystems/quartz/pull/288))
- fix: Use docker default networking
- Update docker to work on macs, update quick start ([#263](https://github.com/informalsystems/quartz/pull/263))

### Misc

- Adding props.onClose() on transfer, deposit, withdraw
  modals ([#270](https://github.com/informalsystems/quartz/pull/270))

---

## Release: v0.1.0

This is the initial release of the quartz framework and CLI.
