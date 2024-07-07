#!/bin/bash
#
# Perform the SessionCreate and SessionSetPubKey handshake between the contract and the sgx node
# Expects:
#   - enclave is already initialized
#   - contract is already deployed
#   - apps/transfers/trusted.hash exists
#

set -eo pipefail

ROOT=${ROOT:-$HOME}

NODE_URL=${NODE_URL:-127.0.0.1:26657}

if [ "$#" -eq 0 ]; then
    echo "Usage: $0 <contract_address>"
    exit 1  # Exit with a non-zero status to indicate an error
fi

CONTRACT="$1" 

CMD="wasmd --node http://$NODE_URL"

cd "$ROOT/cycles-quartz/apps/mtcs"
export TRUSTED_HASH=$(cat trusted.hash)

echo "using CMD: $CMD"
echo "--------------------------------------------------------"

echo "create session"

# change to relay dir
cd $ROOT/cycles-quartz/relayer

# execute SessionCreate on enclave
export EXECUTE_CREATE=$(./scripts/relay.sh SessionCreate)

# submit SessionCreate to contract
RES=$($CMD tx wasm execute "$CONTRACT" "$EXECUTE_CREATE" --from admin --chain-id testing -y --output json)
TX_HASH=$(echo $RES | jq -r '.["txhash"]')

# wait for tx to commit
while ! $CMD query tx $TX_HASH &> /dev/null; do
    echo "... 🕐 waiting for tx"
    sleep 1 
done 

# need to wait another block for light client proof
BLOCK_HEIGHT=$($CMD query block | jq .block.header.height)

echo "at heigh $BLOCK_HEIGHT. need to wait for a block"
while [[ $BLOCK_HEIGHT == $($CMD query block | jq .block.header.height) ]]; do
    echo "... 🕐 waiting for another block"
    sleep 1
done

# need to wait another block for light client proof
BLOCK_HEIGHT=$($CMD query block | jq .block.header.height)
echo "at heigh $BLOCK_HEIGHT. need to wait for a block"
while [[ $BLOCK_HEIGHT == $($CMD query block | jq .block.header.height) ]]; do
    echo "... 🕐 waiting for another block"
    sleep 1
done

echo "--------------------------------------------------------"

echo "set session pk"

# change to prover dir
cd $ROOT/cycles-quartz/utils/tm-prover
export PROOF_FILE="light-client-proof.json"
if [ -f "$PROOF_FILE" ]; then
    rm "$PROOF_FILE"
    echo "removed old $PROOF_FILE"
fi

# TODO: pass this in?
echo "trusted hash $TRUSTED_HASH"
echo "contract $CONTRACT"

# run prover to get light client proof
# TODO: assume this binary is pre-built?
# TODO: pass in addresses and chain id 
cargo run -vvv -- --chain-id testing \
    --primary "http://$NODE_URL" \
    --witnesses "http://$NODE_URL" \
    --trusted-height 500000 \
    --trusted-hash $TRUSTED_HASH \
    --contract-address $CONTRACT \
    --storage-key "quartz_session" \
    --trace-file $PROOF_FILE

export POP=$(cat $PROOF_FILE)
export POP_MSG=$(jq -nc --arg message "$POP" '$ARGS.named')

# execute SessionSetPubKey on enclave
cd $ROOT/cycles-quartz/relayer
export EXECUTE_SETPUB=$(./scripts/relay.sh SessionSetPubKey "$POP_MSG")

RES=$($CMD tx wasm execute "$CONTRACT" "$EXECUTE_SETPUB" --from admin --chain-id testing -y --output json)
TX_HASH=$(echo $RES | jq -r '.["txhash"]')

# wait for tx to commit
while ! $CMD query tx $TX_HASH &> /dev/null; do
    echo "... 🕐 waiting for tx"
    sleep 1 
done 

echo "--------------------------------------------------------"

echo "check session success"
export NONCE_AND_KEY=$($CMD query wasm contract-state raw "$CONTRACT" $(printf '%s' "quartz_session" | hexdump -ve '/1 "%02X"') -o json | jq -r .data | base64 -d)
echo $NONCE_AND_KEY
export PUBKEY=$(echo $NONCE_AND_KEY  | jq -r .pub_key)

