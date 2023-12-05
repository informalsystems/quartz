# CosmWasm smart contract to support MTCS on TEE

An implementation of the on-chain component of
the [Key managers proposal v1](https://github.com/informalsystems/tee-mtcs/issues/26).

## Testing instructions

* Upload a cycle of obligations -

```
export EXECUTE='{
    "join_compute_node": {
        "io_exchange_key": "03E67EF09213633074FB4FBF338643F4F0C574ED60EF11D03422EEB06FA38C8F3F", 
        "address": "wasm10n4dsljyyfp2k2hy6e8vuc9ry32px2egwt5e0m", 
        "nonce": "425d87f8620e1dedeee70590cc55b164b8f01480ee59e0b1da35436a2f7c2777"
    }
}'
wasmd tx wasm execute "$CONTRACT" "$EXECUTE" --from alice --chain-id testing -y
```

* Query requests -

```
wasmd query wasm contract-state smart "$CONTRACT" '{
  "get_requests": { }
}'
```
