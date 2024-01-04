# The Tendermint light client prover

Enables stateless light client verification by generating a light client proof (AKA verification trace) for a given
block height and trusted height/hash.

## Usage

```bash
cargo run -- --chain-id osmosis-1 \
          --primary "http://127.0.0.1:26657" \
          --witnesses "http://127.0.0.1:26657" \
          --trusted-height 1 \
          --trusted-hash "798E237C6FDF39EDA8BA7AB8E8F5DC71F24BC7138BE31882338022F8F88086EE" \
          --contract-address "wasm14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s0phg4d" \
          --storage-key "requests" \
          --trace-file light-client-proof.json
```
