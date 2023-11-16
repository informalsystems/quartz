# The Tendermint light client prover

Enables stateless light client verification by generating a light client proof (AKA verification trace) for a given
block height and trusted height/hash.

## Usage

```bash
cargo run -- --chain-id osmosis-1 \
          --primary https://rpc.osmosis.zone \
          --witnesses https://rpc.osmosis.zone \
          --trusted-height 12230413 \
          --trusted-hash D3742DD1573436AF972472135A24B31D5ACE9A2C04791A76196F875955B90F1D \
          --height 12230423 \
          --trace-file light-client-proof.json
```
