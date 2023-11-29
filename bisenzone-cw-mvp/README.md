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

* Upload a cycle of obligations -

```
export EXECUTE='{
  "upload_obligation": {
    "creditor": "wasm19u72czh0w4jraan8esalv48nrwemh8kgax69yw",
    "amount": "100",
    "memo": "alice -> bob"
  }
}'
wasmd tx wasm execute "$CONTRACT" "$EXECUTE" --from alice --chain-id testing -y

export EXECUTE='{
  "upload_obligation": {
    "creditor": "wasm12r9t5wmre89rwakr0e5nyhfmaf4kdleyltsm9f",
    "amount": "80",
    "memo": "bob -> charlie"
  }
}'
wasmd tx wasm execute "$CONTRACT" "$EXECUTE" --from bob --chain-id testing -y

export EXECUTE='{
  "upload_obligation": {
    "creditor": "wasm19xlctyn7ha6pqg7pk9lnk8y60rk8646dm86qgv",
    "amount": "70",
    "memo": "charlie -> alice"
  }
}'
wasmd tx wasm execute "$CONTRACT" "$EXECUTE" --from charlie --chain-id testing -y
```

* Clear cycle -

```
export EXECUTE='{
  "apply_cycle": {
    "path": ["wasm19xlctyn7ha6pqg7pk9lnk8y60rk8646dm86qgv", "wasm19u72czh0w4jraan8esalv48nrwemh8kgax69yw", "wasm12r9t5wmre89rwakr0e5nyhfmaf4kdleyltsm9f", "wasm19xlctyn7ha6pqg7pk9lnk8y60rk8646dm86qgv"],
    "amount": "70"
  }
}'
wasmd tx wasm execute "$CONTRACT" "$EXECUTE" --from alice --chain-id testing -y
```

* Query obligations -

```
* wasmd query wasm contract-state smart "$CONTRACT" '{
  "get_obligations": {
    "creditor": "wasm19u72czh0w4jraan8esalv48nrwemh8kgax69yw"
  }
}'
```

* Check balance of the solver (to confirm increase in karma) -

```
wasmd query wasm contract-state smart "$CONTRACT" '{
  "balance": {
    "address": "wasm19xlctyn7ha6pqg7pk9lnk8y60rk8646dm86qgv"
  }
}'
```
