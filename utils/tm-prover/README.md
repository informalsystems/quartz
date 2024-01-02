# The Tendermint light client prover

Enables stateless light client verification by generating a light client proof (AKA verification trace) for a given
block height and trusted height/hash.

## Usage

```bash
cargo run -- --chain-id osmosis-1 \
          --primary "http://127.0.0.1:26657" \
          --witnesses "http://127.0.0.1:26657" \
          --trusted-height 400 \
          --trusted-hash "DEA1738C2AEE72E935E39CE6EB8765B8782B791038789AC2FEA446526FDE8638" \
          --contract-address "wasm17p9rzwnnfxcjp32un9ug7yhhzgtkhvl9jfksztgw5uh69wac2pgsm0v070" \
          --storage-key "requests" \
          --trace-file light-client-proof.json
```
