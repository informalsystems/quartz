# Obligato web3 liquidity demo

This demo shows end-to-end integration with Obligato for web3 liquidity (i.e. a native ERC20 token).

This demo is expected to run on a single machine that has SGX.

It depends on `wasmd` (version v0.44.0). Follow the instructions [here](https://docs.cosmwasm.com/docs/getting-started/installation/#wasmd) to install, but checkout version `v0.44.0`. 

## Create obligations and tenders on Obligato

Make sure tenders have backing funds.

## Start blockchain

```
# cd bisenzone-cw-mvp

./scripts/keygen.sh
./scripts/init-node.sh
./scripts/run-node.sh
```

## Build contract

```
./scripts/build-contract.sh
```

## Listen to events (for debugging)

```
websocat ws://127.0.0.1:26657/websocket
{ "jsonrpc": "2.0", "method": "subscribe", "params": ["tm.event='Tx'"], "id": 1 }
```

## Init enclave

### Setup and build

Generate the private key and build the binary that will run in the enclave:

```
# cd tee-mtcs/enclaves/quartz

gramine-sgx-gen-private-key

CARGO_TARGET_DIR=./target cargo build --release
```

The built binary is a grpc server that hosts the (currently built-in) mtcs application. 

### Update enclave trusted hash

Now we need to get the trusted hash to initialize the enclave. Running tm-prover with wrong trusted-hash should print the correct one. 

```
# cd tee-mtcs/utils/tm-prover

rm light-client-proof.json
cargo run -- --chain-id testing \
--primary "http://127.0.0.1:26657" \
--witnesses "http://127.0.0.1:26657" \
--trusted-height 1 \
--trusted-hash "5237772462A41C0296ED688A0327B8A60DF310F08997AD760EB74A70D0176C27" \
--contract-address "wasm14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s0phg4d" \
--storage-key "quartz_session" \
--trace-file light-client-proof.json &> output
cat output | grep found | head -1 | awk '{print $NF}' | sed 's/\x1b\[[0-9;]*m//g' > trusted.hash
export TRUSTED_HASH=$(cat trusted.hash)
```

Note we dump the output of the command into the `output` file, which we then parse to get the trusted hash,
strip of any extra chars, and finally save into the `trusted.hash` file (we'll use this again laster). We also save it to an env var.


### Start enclave

Update the `quartz-manifest.template` with the correct ("found") hash from the previous command:

```
# cd tee-mtcs/enclaves/quartz

sed -i -r "s/(\"--trusted-hash\", \")[A-Z0-9]+(\"])/\1$TRUSTED_HASH\2/" quartz.manifest.template
```

That will overwrite the template file in place, inserting the new hash in place of the old one. 

Now we can start the enclave:

```
# cd tee-mtcs/enclaves/quartz

gramine-manifest  \
-Dlog_level="error"  \
-Dhome=${HOME}  \
-Darch_libdir="/lib/$(gcc -dumpmachine)"  \
-Dra_type="epid" \
-Dra_client_spid="51CAF5A48B450D624AEFE3286D314894" \
-Dra_client_linkable=1 \
-Dquartz_dir="$(pwd)"  \
quartz.manifest.template quartz.manifest

gramine-sgx-sign --manifest quartz.manifest --output quartz.manifest.sgx
gramine-sgx ./quartz
```

## Send initiate request

Now with the binary running in the enclave, we can run commands in another window.

First, let's instantiate:

```
# cd tee-mtcs/utils/quartz-relayer

export INSTANTIATE_MSG=$(./scripts/relay.sh Instantiate)
```

Note we save the output into an env variable.

## Deploy contract

We can now deploy the contract. The deploy script will read from $INSTANTIATE_MSG and use the attestation to create the contract.

```
# cd bisenzone-cw-mvp

./scripts/deploy-contract.sh artifacts/cw_tee_mtcs.wasm &> output

export CONTRACT=$(cat output | grep Address | awk '{print $NF}' | sed 's/\x1b\[[0-9;]*m//g')
```

Note again we save the output to a file, and then parse the file to get the contract address, which we save in an env var.

## Create session

Now we can initialize a session on the enclave, which will generate a nonce to use:

```
# cd tee-mtcs/utils/quartz-relayer

export EXECUTE=$(./scripts/relay.sh SessionCreate)
```

And we can execute the session creation on the contract:

```
wasmd tx wasm execute "$CONTRACT" "$EXECUTE" --from alice --chain-id testing -y
```

## Set session pk

Now let's generate a light client proof that the contract has created the session:

```
# cd tee-mtcs/utils/tm-prover

rm light-client-proof.json
export TRUSTED_HASH=$(cat trusted.hash)
cargo run -- --chain-id testing \
--primary "http://127.0.0.1:26657" \
--witnesses "http://127.0.0.1:26657" \
--trusted-height 1 \
--trusted-hash $TRUSTED_HASH \
--contract-address $CONTRACT \
--storage-key "quartz_session" \
--trace-file light-client-proof.json
```

And store the proof in an env var:

```
export POP=$(cat light-client-proof.json)
export POP_MSG=$(jq -nc --arg message "$POP" '$ARGS.named')
```

Now we can relay this proof to the enclave, so it can attest to the pubkey:

```
# cd tee-mtcs/utils/quartz-relayer

export EXECUTE=$(./scripts/relay.sh SessionSetPubKey "$POP_MSG")
```

And send the attestation back to the contract:

```
wasmd tx wasm execute "$CONTRACT" "$EXECUTE" --from alice --chain-id testing -y
```

## Check for session success

Wait a few seconds for the tx to commit, and then fetch the nonce and pubkey data:

```
export NONCE_AND_KEY=$(wasmd query wasm contract-state raw "$CONTRACT" $(printf '%s' "quartz_session" | hexdump -ve '/1 "%02X"') -o json | jq -r .data | base64 -d)
echo $NONCE_AND_KEY
# Note if you see an empty pubkey, wait a few seconds and rerun the above wasmd query command
# {"nonce":"d3283ed5d646298c27f5ef1726c42bf4853ed7f3d30c905fd3607ecc56903db4","pub_key":"02e4d8bc80d032ad610e4643c3da4235076b4d24335cc4c77592562bdcd62ce1d0"}

export PUBKEY=$(echo $NONCE_AND_KEY  | jq -r .pub_key)
```

## Sync obligations

Now with the pubkey in hand, we can fetch the obligations from Obligato and encrypt them:

```
# cd tee-mtcs/utils/cycles-sync

cargo run -- --keys-file keys.json \
            --obligation-user-map-file o_map.json \
            --user "alice" \
            --contract $CONTRACT \
            sync-obligations \
            --epoch-pk $PUBKEY

export OBLIGATIONS=$(wasmd query wasm contract-state raw "$CONTRACT" $(printf '%s' "1/obligations" | hexdump -ve '/1 "%02X"') -o json | jq -r .data | base64 -d)
```

## Init clearing

Create a clearing cycle on Obligato (required to be able to upload setoffs to Obligato) and initiate clearing on the
blockchain.

```
wasmd tx wasm execute $CONTRACT '"init_clearing"' --from alice --chain-id testing -y
```

## Run clearing on enclave

With the encrypted obligations in hand, and clearing run initiated on chain, we can run clearing on the enclave:

```
# cd tee-mtcs/enclaves/quartz

export REQUEST_MSG=$(jq -nc --arg message "$OBLIGATIONS" '$ARGS.named')

export SETOFFS=$(grpcurl -plaintext -import-path ../../enclaves/quartz/proto/ -proto mtcs.proto -d "$REQUEST_MSG" '127.0.0.1:11090' mtcs.Clearing/Run | jq -c '.message | fromjson')
```

## Submit setoffs

```
export EXECUTE=$(jq -nc --argjson submit_setoffs "$SETOFFS" '$ARGS.named')
wasmd tx wasm execute "$CONTRACT" "$EXECUTE" --from alice --chain-id testing -y --gas 2000000

wasmd query wasm contract-state raw "$CONTRACT" $(printf '%s' "1/setoffs" | hexdump -ve '/1 "%02X"') -o json | jq -r .data | base64 -d
```

## Verify CW20 balances

TODO: replace addresses

```
wasmd query wasm contract-state smart "$CONTRACT" '{"balance": {"address": "wasm1gjg72awjl7jvtmq4kjqp3al9p6crstpar8wgn5"}}'
wasmd query wasm contract-state smart "$CONTRACT" '{"balance": {"address": "wasm1tawlwmllmnwm950a7uttqlyne3k4774rsnuw6e"}}'
```

## Sync setoffs

```
# cd tee-mtcs/utils/cycles-sync

cargo run -- --keys-file keys.json \
            --obligation-user-map-file o_map.json \
            --user "alice" \
            --contract $CONTRACT \
            sync-set-offs
```
