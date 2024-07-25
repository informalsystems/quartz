#!/bin/bash

set -eo pipefail
ROOT=${HOME}

WASMD_HOME=${WASMD_HOME:-"/home/peppi/.neutrond"}
USER_ADDR=$(neutrond keys show -a "val1" --keyring-backend test --home "$WASMD_HOME" --keyring-dir "/home/peppi/.neutrond/")

if [ -z "$USER_ADDR" ]; then
    echo "❌ Error: User address not found. Please ensure the key exists in the keyring."
    exit 1
fi

WASM_BIN="$1"
INSTANTIATE_MSG="$2"
CHAIN_ID=${CHAIN_ID:-test-1}
NODE_URL=${NODE_URL:-127.0.0.1:26657}
LABEL=${LABEL:-quartz-transfers-app}
COUNT=${COUNT:-0}

# Use the QUARTZ_PORT environment variable if set, otherwise default to 11090
QUARTZ_PORT="${QUARTZ_PORT:-11090}"

TXFLAG="--chain-id ${CHAIN_ID} --gas-prices 0.0025untrn --gas auto --gas-adjustment 1.3"

CMD="neutrond --node http://$NODE_URL"

echo "🚀 Deploying WASM contract '${WASM_BIN}' on chain '${CHAIN_ID}' using account '${USER_ADDR}'..."
echo " with cmd : $CMD"
echo "===================================================================="

echo "Storing WASM contract..."
RES=$($CMD tx wasm store "$WASM_BIN" --from "$USER_ADDR" --keyring-backend "test" $TXFLAG -y --output json --keyring-dir "/home/peppi/.neutrond/")
echo "Store transaction result:"
# echo "$RES" | jq '.'
TX_HASH=$(echo "$RES" | jq -r '.txhash')
echo "Transaction hash: $TX_HASH"

echo "Waiting for transaction to be included in a block..."
ATTEMPTS=0
MAX_ATTEMPTS=30
while [ $ATTEMPTS -lt $MAX_ATTEMPTS ]; do
    TX_RESULT=$($CMD query tx "$TX_HASH" --output json 2>/dev/null || echo '{"code": 1}')
    TX_CODE=$(echo "$TX_RESULT" | jq -r '.code // .tx_result.code // 1')
    if [[ $TX_CODE == "0" ]]; then
        echo "Transaction processed successfully."
        break
    elif [[ $TX_CODE != "1" ]]; then
        echo "Error processing transaction. Code: $TX_CODE"
        echo "Full transaction result:"
        # echo "$TX_RESULT" | jq '.'
        exit 1
    fi
    echo "Transaction not yet processed. Waiting... (Attempt $((ATTEMPTS+1))/$MAX_ATTEMPTS)"
    sleep 2
    ATTEMPTS=$((ATTEMPTS+1))
done

if [ $ATTEMPTS -eq $MAX_ATTEMPTS ]; then
    echo "Failed to retrieve transaction after $MAX_ATTEMPTS attempts. Last response:"
    # echo "$TX_RESULT" | jq '.'
    exit 1
fi

echo "Transaction result:"
# echo "$TX_RESULT" | jq '.'

echo "Extracting CODE_ID..."
CODE_ID=$(echo "$TX_RESULT" | jq -r '.events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')
echo "Extracted CODE_ID: $CODE_ID"

if [[ -z "$CODE_ID" || "$CODE_ID" == "null" ]]; then
    echo "Failed to extract CODE_ID. Printing full transaction result for debugging:"
    echo "$TX_RESULT" | jq '.'
    exit 1
fi

echo ""
echo "🚀 Instantiating contract with the following parameters:"
echo "--------------------------------------------------------"
echo "Label: ${LABEL}"
echo "CODE_ID: ${CODE_ID}"
# echo "INSTANTIATE_MSG: ${INSTANTIATE_MSG}"
echo "--------------------------------------------------------"

# Parse and re-format the INSTANTIATE_MSG
INSTANTIATE_MSG_PARSED=$(echo "$INSTANTIATE_MSG" | jq -r '.')
INSTANTIATE_MSG_ONELINE=$(echo "$INSTANTIATE_MSG_PARSED" | jq '{quartz: .} + {denom: "untrn"}'  )

echo "Parsed INSTANTIATE_MSG:"
# echo "$INSTANTIATE_MSG_PARSED" | jq '.'

INSTANTIATE_CMD="$CMD tx wasm instantiate $CODE_ID '$INSTANTIATE_MSG_ONELINE' --from $USER_ADDR --keyring-backend test --keyring-dir /home/peppi/.neutrond/ --label $LABEL $TXFLAG -y --no-admin --output json"
echo "Instantiate command:"
# echo "$INSTANTIATE_CMD"
echo "--------------------------------------------------------"

echo "Executing instantiate command..."
RES=$(eval "$INSTANTIATE_CMD")
echo "Instantiate result:"
# echo "$RES" | jq '.'
TX_HASH=$(echo "$RES" | jq -r '.txhash')

echo ""
echo "Waiting for instantiate transaction to be processed..."
ATTEMPTS=0
while [ $ATTEMPTS -lt $MAX_ATTEMPTS ]; do
    TX_RESULT=$($CMD query tx "$TX_HASH" --output json 2>/dev/null || echo '{"code": 1}')
    TX_CODE=$(echo "$TX_RESULT" | jq -r '.code // .tx_result.code // 1')
    if [[ $TX_CODE == "0" ]]; then
        echo "Instantiate transaction processed successfully."
        break
    elif [[ $TX_CODE != "1" ]]; then
        echo "Error processing instantiate transaction. Code: $TX_CODE"
        echo "Full transaction result:"
        # echo "$TX_RESULT" | jq '.'
        exit 1
    fi
    echo "Instantiate transaction not yet processed. Waiting... (Attempt $((ATTEMPTS+1))/$MAX_ATTEMPTS)"
    sleep 2
    ATTEMPTS=$((ATTEMPTS+1))
done

if [ $ATTEMPTS -eq $MAX_ATTEMPTS ]; then
    echo "Failed to retrieve instantiate transaction after $MAX_ATTEMPTS attempts. Last response:"
    echo "$TX_RESULT" | jq '.'
    exit 1
fi

echo "Querying for instantiated contract..."
RES=$($CMD query wasm list-contract-by-code "$CODE_ID" --output json)
echo "Query result:"
# echo "$RES" | jq '.'
CONTRACT=$(echo "$RES" | jq -r '.contracts[0]')

if [[ -z "$CONTRACT" || "$CONTRACT" == "null" ]]; then
    echo "Failed to retrieve contract address. Printing full query result for debugging:"
    echo "$RES" | jq '.'
    exit 1
fi

echo "CONTRACT: $CONTRACT"

cd $ROOT/cycles-quartz/relayer

# execute SessionCreate on enclave
echo "Executing SessionCreate on enclave..."
export EXECUTE_CREATE=$(QUARTZ_PORT=$QUARTZ_PORT ./scripts/relayNeutron.sh SessionCreate)
if [ -z "$EXECUTE_CREATE" ]; then
    echo "❌ Error: Failed to execute SessionCreate on enclave"
    exit 1
fi
echo "SessionCreate execution successful"

# submit SessionCreate to contract
echo "Submitting SessionCreate to contract..."
RES=$($CMD tx wasm execute "$CONTRACT" "$EXECUTE_CREATE" --from "$USER_ADDR"  $TXFLAG --keyring-backend test --keyring-dir "/home/peppi/.neutrond/" --chain-id test-1 -y --output json)
echo "SessionCreate submission result:"
# echo "$RES" | jq '.'

TX_HASH=$(echo "$RES" | jq -r '.txhash')
if [ -z "$TX_HASH" ] || [ "$TX_HASH" == "null" ]; then
    echo "❌ Error: Failed to retrieve transaction hash"
    exit 1
fi
echo "Transaction hash: $TX_HASH"

# wait for tx to commit
echo "Waiting for transaction to commit..."
ATTEMPTS=0
MAX_ATTEMPTS=30
while [ $ATTEMPTS -lt $MAX_ATTEMPTS ]; do
    if $CMD query tx "$TX_HASH" &> /dev/null; then
        echo "✅ Transaction committed successfully"
        break
    fi
    echo "... 🕐 waiting for tx (Attempt $((ATTEMPTS+1))/$MAX_ATTEMPTS)"
    sleep 2
    ATTEMPTS=$((ATTEMPTS+1))
done

if [ $ATTEMPTS -eq $MAX_ATTEMPTS ]; then
    echo "❌ Error: Transaction failed to commit after $MAX_ATTEMPTS attempts"
    exit 1
fi

# wait for two blocks
echo "Waiting for two additional blocks..."
INITIAL_HEIGHT=$($CMD status | jq -r '.sync_info.latest_block_height')
TARGET_HEIGHT=$((INITIAL_HEIGHT + 2))

while true; do
    CURRENT_HEIGHT=$($CMD status | jq -r '.sync_info.latest_block_height')
    if [ "$CURRENT_HEIGHT" -ge "$TARGET_HEIGHT" ]; then
        echo "✅ Two additional blocks have been produced."
        break
    fi
    echo "Current height: $CURRENT_HEIGHT, waiting for height: $TARGET_HEIGHT"
    sleep 2
done

echo "✅ Handshake process completed"


echo "--------------------------------------------------------"
echo "set session pk"

# change to prover dir
cd $ROOT/cycles-quartz/utils/tm-prover
export PROOF_FILE="light-client-proof.json"
rm -f "$PROOF_FILE"

echo "removed old $PROOF_FILE"

echo "--------------------------------------------------------"
echo "Waiting for new blocks to be produced..."

# wait for two blocks
echo "Waiting for two additional blocks..."
INITIAL_HEIGHT=$($CMD status | jq -r '.sync_info.latest_block_height')
TARGET_HEIGHT=$((INITIAL_HEIGHT + 2))

while true; do
    CURRENT_HEIGHT=$($CMD status | jq -r '.sync_info.latest_block_height')
    if [ "$CURRENT_HEIGHT" -ge "$TARGET_HEIGHT" ]; then
        echo "✅ Two additional blocks have been produced."
        break
    fi
    echo "Current height: $CURRENT_HEIGHT, waiting for height: $TARGET_HEIGHT"
    sleep 2
done
echo "Required blocks produced. Proceeding with tm-prover..."

# Get the latest block for trusted height and hash
#LATEST_BLOCK=$($CMD status )
#export TRUSTED_HEIGHT=$(echo $LATEST_BLOCK |  jq -r '.sync_info.latest_block_height')
# export TRUSTED_HASH=$(echo $LATEST_BLOCK |  jq -r '.sync_info.latest_app_hash')

cd "$HOME/cycles-quartz/apps/transfers"
export TRUSTED_HASH=$(cat trusted.hash)
export TRUSTED_HEIGHT=$(cat trusted.height)

echo "trusted hash $TRUSTED_HASH"
echo "contract $CONTRACT"

cd $ROOT/cycles-quartz/utils/tm-prover
export QUARTZ_SESSION=$($CMD query wasm contract-state raw $CONTRACT $(echo -n "quartz_session" | xxd -p -c 20) --node "http://$NODE_URL")
echo "Quartz Session before prover: $QUARTZ_SESSION"
echo "trusted height $TRUSTED_HEIGHT"



export PROOF_FILE="light-client-proof.json"
if [ -f "$PROOF_FILE" ]; then
    rm "$PROOF_FILE"
    echo "removed old $PROOF_FILE"
fi

# run prover to get light client proof
cargo run -- --chain-id test-1 \
    --primary "http://$NODE_URL" \
    --witnesses "http://$NODE_URL" \
    --trusted-height $TRUSTED_HEIGHT \
    --trusted-hash $TRUSTED_HASH \
    --contract-address $CONTRACT \
    --storage-key "quartz_session" \
    --trace-file $PROOF_FILE

export POP=$(cat $PROOF_FILE)
export POP_MSG=$(jq -nc --arg message "$POP" '$ARGS.named')

echo "DEBUG: POP_MSG content:"
# echo "$POP_MSG" | jq .

# execute SessionSetPubKey on enclave
cd $ROOT/cycles-quartz/relayer
export EXECUTE_SETPUB=$(QUARTZ_PORT=$QUARTZ_PORT ./scripts/relayNeutron.sh SessionSetPubKey "$POP_MSG")

RES=$($CMD tx wasm execute "$CONTRACT" "$EXECUTE_SETPUB" --from "$USER_ADDR"  $TXFLAG --keyring-backend test --keyring-dir "/home/peppi/.neutrond/" --chain-id test-1 -y --output json)
TX_HASH=$(echo $RES | jq -r '.txhash')


if [ -z "$TX_HASH" ] || [ "$TX_HASH" == "null" ]; then
    echo "❌ Error: Failed to retrieve transaction hash"
    exit 1
fi
echo "Transaction hash: $TX_HASH"

# wait for tx to commit
echo "Waiting for transaction to commit..."
ATTEMPTS=0
MAX_ATTEMPTS=30
while [ $ATTEMPTS -lt $MAX_ATTEMPTS ]; do
    if $CMD query tx "$TX_HASH" &> /dev/null; then
        echo "✅ Transaction committed successfully"
        break
    fi
    echo "... 🕐 waiting for tx (Attempt $((ATTEMPTS+1))/$MAX_ATTEMPTS)"
    sleep 2
    ATTEMPTS=$((ATTEMPTS+1))
done

if [ $ATTEMPTS -eq $MAX_ATTEMPTS ]; then
    echo "❌ Error: Transaction failed to commit after $MAX_ATTEMPTS attempts"
    exit 1
fi

echo "--------------------------------------------------------"
echo "check session success"
export NONCE_AND_KEY=$($CMD query wasm contract-state raw "$CONTRACT" $(printf '%s' "quartz_session" | hexdump -ve '/1 "%02X"') -o json | jq -r .data | base64 -d)
echo $NONCE_AND_KEY
export PUBKEY=$(echo $NONCE_AND_KEY  | jq -r .pub_key)

echo "🚀 Deployment and handshake completed successfully!"
echo "📌 Contract Address: ${CONTRACT}"
echo "🔑 Contract Key: ${PUBKEY}"