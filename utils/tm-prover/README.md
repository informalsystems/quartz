# The Tendermint light client prover

Enables stateless light client verification by generating a light client proof (AKA verification trace) for a given
block height and trusted height/hash.

## Usage

```bash
cargo run -- --chain-id testing \
          --primary "http://127.0.0.1:26657" \
          --witnesses "http://127.0.0.1:26657" \
          --trusted-height 500000 \
          --trusted-hash "2EF0E6F9BDDF5DEAA6FCD6492C3DB26D7C62BFFC01B538A958D04376E0B67185" \
          --contract-address "wasm14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s0phg4d" \
          --storage-key "quartz_session" \
          --trace-file light-client-proof.json
```
