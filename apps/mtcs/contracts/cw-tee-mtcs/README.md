# CosmWasm smart contract to support MTCS on TEE

An implementation of the on-chain component of
the [Key managers proposal v1](https://github.com/informalsystems/tee-mtcs/issues/26).

## Testing instructions

* Submit a bootstrap key manager request -

```
export EXECUTE='{
    "bootstrap_key_manager": {
        "compute_mrenclave": "dc43f8c42d8e5f52c8bbd68f426242153f0be10630ff8cca255129a3ca03d273", 
        "key_manager_mrenclave": "1cf2e52911410fbf3f199056a98d58795a559a2e800933f7fcd13d048462271c", 
        "tcb_info": ""
    }
}'
wasmd tx wasm execute "$CONTRACT" "$EXECUTE" --from alice --chain-id testing -y
```

* Query the bootstrap state -

```
wasmd query wasm contract-state raw "$CONTRACT" 7367787374617465 # BIN_HEX('sgx_state')
# OR ----
wasmd query wasm contract-state smart "$CONTRACT" '{
  "get_sgx_state": { }
}'
```

* Submit a join compute node request -

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
