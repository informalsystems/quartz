# Bisenzone CosmWasm MVP

CosmWasm smart contracts used in the Bisenzone MVP.

## Testing instructions

* Deploy the smart contract on a local wasmd instance
  (see [Obligation Clearing Mvp: Local setup](https://github.com/informalsystems/obligation-clearing-mvp#local-setup))
    * Should normally be as simple as running the following scripts ->
    ```bash
    # terminal-1
    ./scripts/keygen.sh
    ./scripts/init-node.sh
    ./scripts/run-node.sh
    ```
    ```bash
    # terminal-2
    ./scripts/build-contract.sh
    ./scripts/deploy-contract.sh artifacts/cofi_karma_game.wasm
    ```
* Set contract env var (using the output of the `deploy.sh` script) -

```
export CONTRACT="wasm13we0myxwzlpx8l5ark8elw5gj5d59dl6cjkzmt80c5q5cv5rt54qhmta7s"
```

For further instructions, refer to the contract specific READMEs
(e.g. [cofi-karma-game README.md](contracts/cofi-karma-game/README.md)) 