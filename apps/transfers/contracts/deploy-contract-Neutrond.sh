# # # #!/bin/bash

# # # # Deploy the specified contract's `WASM_BIN` to the chain specified by `CHAIN_ID` using the `USER_ADDR` account.

# # # set -eo pipefail

# # # usage() {
# # #     echo "Usage: $0 WASM_BIN [COUNT]"
# # #     echo "Example: $0 artifacts/cofi_karma_game.wasm"
# # #     exit 1
# # # }

# # # if [ -z "$1" ]; then
# # #     echo "❌ Error: Missing WASM_BIN parameter. Please check if all parameters were specified."
# # #     usage
# # # fi

# # # if [ "$#" -gt 2 ]; then
# # #     echo "❌ Error: Incorrect number of parameters."
# # #     usage
# # # fi

# # # # WASMD_HOME=${WASMD_HOME:-"$ROOT/.neutrond"}
# # # USER_ADDR=${USER_ADDR:-$(neutrond keys show -a "val1" --keyring-backend "test" --keyring-dir "/home/peppi/.neutrond/")}
# # # # USER_ADDR=$(neutrond keys show -a demowallet1 --keyring-backend test --home ${WASMD_HOME})

# # # WASM_BIN="$1"
# # # CHAIN_ID=${CHAIN_ID:-test-1}
# # # NODE_URL=${NODE_URL:-http://127.0.0.1:26657}
# # # LABEL=${LABEL:-bisenzone-mvp}
# # # COUNT=${COUNT:-0}
# # # INSTANTIATE_MSG=${INSTANTIATE_MSG:-"{}"}

# # # TXFLAG="--chain-id ${CHAIN_ID} --gas-prices 0.0025untrn --gas auto --gas-adjustment 1.3"

# # # CMD="neutrond --node $NODE_URL"

# # # echo "🚀 Deploying WASM contract '${WASM_BIN}' on chain '${CHAIN_ID}' using account '${USER_ADDR}'..."
# # # echo " with cmd : $CMD"
# # # echo "===================================================================="

# # # RES=$($CMD tx wasm store "$WASM_BIN" --from $USER_ADDR $TXFLAG -y --output json)
# # # echo $RES
# # # TX_HASH=$(echo $RES | jq -r '.["txhash"]')

# while ! $CMD query tx $TX_HASH &> /dev/null; do
#     echo "... 🕐 waiting for contract to deploy from tx hash $TX_HASH"
#     sleep 1
# done

# RES=$($CMD query tx "$TX_HASH" --output json)
# CODE_ID=$(echo $RES | jq -r '.logs[0].events[1].attributes[0].value')

# # # echo ""
# # # echo "🚀 Instantiating contract with the following parameters:"
# # # echo "--------------------------------------------------------"
# # # echo "Label: ${LABEL}"
# # # echo "--------------------------------------------------------"

# # # RES=$($CMD tx wasm instantiate "$CODE_ID" "$INSTANTIATE_MSG" --from $USER_ADDR --label $LABEL $TXFLAG -y --no-admin --output json)
# # # TX_HASH=$(echo $RES | jq -r '.["txhash"]')

# # # echo ""
# # # while ! $CMD query tx $TX_HASH &> /dev/null; do
# # #     echo "... 🕐 waiting for contract to be queryable"
# # #     sleep 1
# # # done

# # # RES=$($CMD query wasm list-contract-by-code "$CODE_ID" --output json)
# # # CONTRACT=$(echo $RES | jq -r '.contracts[0]')

# # # echo "🚀 Successfully deployed and instantiated contract!"
# # # echo "🔗 Chain ID: ${CHAIN_ID}"
# # # echo "🆔 Code ID: ${CODE_ID}"
# # # echo "📌 Contract Address: ${CONTRACT}"
# # # echo "🔖 Contract Label: ${LABEL}"
# # #!/bin/bash

# # # Deploy the specified contract's `WASM_BIN` to the chain specified by `CHAIN_ID` using the `USER_ADDR` account.

# # set -eo pipefail



# # WASMD_HOME=${WASMD_HOME:-"/home/peppi/.neutrond"}
# # USER_ADDR=$(neutrond keys show -a "val1" --keyring-backend test --home "$WASMD_HOME" --keyring-dir "/home/peppi/.neutrond/")

# # if [ -z "$USER_ADDR" ]; then
# #     echo "❌ Error: User address not found. Please ensure the key exists in the keyring."
# #     exit 1
# # fi

# # WASM_BIN="$1"
# # INSTANTIATE_MSG="$2"
# # CHAIN_ID=${CHAIN_ID:-test-1}
# # NODE_URL=${NODE_URL:-http://127.0.0.1:26657}
# # LABEL=${LABEL:-bisenzone-mvp}
# # COUNT=${COUNT:-0}





# # TXFLAG="--chain-id ${CHAIN_ID} --gas-prices 0.0025untrn --gas auto --gas-adjustment 1.3"

# # CMD="neutrond --node $NODE_URL"

# # echo "🚀 Deploying WASM contract '${WASM_BIN}' on chain '${CHAIN_ID}' using account '${USER_ADDR}'..."
# # echo " with cmd : $CMD"
# # echo "===================================================================="

# # RES=$($CMD tx wasm store "$WASM_BIN" --from "$USER_ADDR" --keyring-backend "test"  $TXFLAG -y --output json --keyring-dir "/home/peppi/.neutrond/")
# # echo $RES
# # TX_HASH=$(echo $RES | jq -r '.["txhash"]')

# # while ! $CMD query tx $TX_HASH &> /dev/null; do
# #     echo "... 🕐 waiting for contract to deploy from tx hash $TX_HASH"
# #     sleep 1
# # done

# # RES=$($CMD query tx "$TX_HASH" --output json)
# # CODE_ID=$(echo $RES | jq -r '.logs[0].events[1].attributes[0].value')

# # echo ""
# # echo "--------------------------------------------------------"
# # echo "Label: ${LABEL}"
# # echo "Received INSTANTIATE_MSG: ${INSTANTIATE_MSG}"
# # echo "--------------------------------------------------------"

# # # Unescape the JSON before passing it to the command
# # UNESCAPED_INSTANTIATE_MSG=$(echo "$INSTANTIATE_MSG" | jq -r '.')

# # echo "Unescaped INSTANTIATE_MSG: ${UNESCAPED_INSTANTIATE_MSG}"
# # echo "--------------------------------------------------------"

# # INSTANTIATE_CMD="$CMD tx wasm instantiate $CODE_ID '$UNESCAPED_INSTANTIATE_MSG' --from $USER_ADDR --keyring-backend test --keyring-dir /home/peppi/.neutrond/ --label $LABEL $TXFLAG -y --no-admin --output json"
# # echo "Instantiate command:"
# # echo "$INSTANTIATE_CMD"
# # echo "--------------------------------------------------------"

# # RES=$(eval "$INSTANTIATE_CMD")
# # echo "Instantiate result:"
# # echo "$RES"
# # TX_HASH=$(echo $RES | jq -r '.["txhash"]')

# # echo ""
# # while ! $CMD query tx $TX_HASH &> /dev/null; do
# #     echo "... 🕐 waiting for contract to be queryable"
# #     sleep 1
# # done

# # RES=$($CMD query wasm list-contract-by-code "$CODE_ID" --output json)
# # CONTRACT=$(echo $RES | jq -r '.contracts[0]')

# # echo "🚀 Successfully deployed and instantiated contract!"
# # echo "🔗 Chain ID: ${CHAIN_ID}"
# # echo "🆔 Code ID: ${CODE_ID}"
# # echo "📌 Contract Address: ${CONTRACT}"
# # echo "🔖 Contract Label: ${LABEL}"


# #!/bin/bash

# set -eo pipefail

# WASMD_HOME=${WASMD_HOME:-"/home/peppi/.neutrond"}
# USER_ADDR=$(neutrond keys show -a "val1" --keyring-backend test --home "$WASMD_HOME" --keyring-dir "/home/peppi/.neutrond/")

# if [ -z "$USER_ADDR" ]; then
#     echo "❌ Error: User address not found. Please ensure the key exists in the keyring."
#     exit 1
# fi

# WASM_BIN="$1"
# INSTANTIATE_MSG="$2"
# CHAIN_ID=${CHAIN_ID:-test-1}
# NODE_URL=${NODE_URL:-http://127.0.0.1:26657}
# LABEL=${LABEL:-bisenzone-mvp}
# COUNT=${COUNT:-0}

# TXFLAG="--chain-id ${CHAIN_ID} --gas-prices 0.0025untrn --gas auto --gas-adjustment 1.3"

# CMD="neutrond --node $NODE_URL"

# echo "🚀 Deploying WASM contract '${WASM_BIN}' on chain '${CHAIN_ID}' using account '${USER_ADDR}'..."
# echo " with cmd : $CMD"
# echo "===================================================================="

# RES=$($CMD tx wasm store "$WASM_BIN" --from "$USER_ADDR" --keyring-backend "test" $TXFLAG -y --output json --keyring-dir "/home/peppi/.neutrond/")
# echo $RES
# TX_HASH=$(echo $RES | jq -r '.["txhash"]')

# echo "Waiting for transaction to be included in a block..."
# while true; do
#     TX_RESULT=$($CMD query tx $TX_HASH --output json 2>/dev/null || echo '{"code": 1}')
#     TX_CODE=$(echo $TX_RESULT | jq -r '.code // 1')
#     if [[ $TX_CODE == "0" ]]; then
#         echo "Transaction processed successfully."
#         break
#     elif [[ $TX_CODE != "1" ]]; then
#         echo "Error processing transaction. Code: $TX_CODE"
#         exit 1
#     fi
#     echo "Transaction not yet processed. Waiting..."
#     sleep 1
# done

# echo "Transaction result:"
# echo "$TX_RESULT"

# # Extract CODE_ID from the transaction result
# CODE_ID=$(echo $TX_RESULT | jq -r '.logs[0].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')

# if [[ -z "$CODE_ID" || "$CODE_ID" == "null" ]]; then
#     echo "Failed to extract CODE_ID. Printing full transaction result for debugging:"
#     echo "$TX_RESULT" | jq '.'
#     exit 1
# fi

# echo "Extracted CODE_ID: $CODE_ID"

# echo ""
# echo "🚀 Instantiating contract with the following parameters:"
# echo "--------------------------------------------------------"
# echo "Label: ${LABEL}"
# echo "CODE_ID: ${CODE_ID}"
# echo "INSTANTIATE_MSG: ${INSTANTIATE_MSG}"
# echo "--------------------------------------------------------"

# # Remove newlines from INSTANTIATE_MSG
# INSTANTIATE_MSG_ONELINE=$(echo "$INSTANTIATE_MSG" | jq -c '.')

# INSTANTIATE_CMD="$CMD tx wasm instantiate $CODE_ID '$INSTANTIATE_MSG_ONELINE' --from $USER_ADDR --keyring-backend test --keyring-dir /home/peppi/.neutrond/ --label $LABEL $TXFLAG -y --no-admin --output json"
# echo "Instantiate command:"
# echo "$INSTANTIATE_CMD"
# echo "--------------------------------------------------------"

# RES=$(eval "$INSTANTIATE_CMD")
# echo "Instantiate result:"
# echo "$RES"
# TX_HASH=$(echo $RES | jq -r '.["txhash"]')

# echo ""
# while ! $CMD query tx $TX_HASH &> /dev/null; do
#     echo "... 🕐 waiting for contract to be queryable"
#     sleep 1
# done

# RES=$($CMD query wasm list-contract-by-code "$CODE_ID" --output json)
# CONTRACT=$(echo $RES | jq -r '.contracts[0]')

# echo "🚀 Successfully deployed and instantiated contract!"
# echo "🔗 Chain ID: ${CHAIN_ID}"
# echo "🆔 Code ID: ${CODE_ID}"
# echo "📌 Contract Address: ${CONTRACT}"
# echo "🔖 Contract Label: ${LABEL}"
#!/bin/bash

set -eo pipefail

WASMD_HOME=${WASMD_HOME:-"/home/peppi/.neutrond"}
USER_ADDR=$(neutrond keys show -a "val1" --keyring-backend test --home "$WASMD_HOME" --keyring-dir "/home/peppi/.neutrond/")

if [ -z "$USER_ADDR" ]; then
    echo "❌ Error: User address not found. Please ensure the key exists in the keyring."
    exit 1
fi

WASM_BIN="$1"
INSTANTIATE_MSG="$2"
CHAIN_ID=${CHAIN_ID:-test-1}
NODE_URL=${NODE_URL:-http://127.0.0.1:26657}
LABEL=${LABEL:-bisenzone-mvp}
COUNT=${COUNT:-0}

TXFLAG="--chain-id ${CHAIN_ID} --gas-prices 0.0025untrn --gas auto --gas-adjustment 1.3"

CMD="neutrond --node $NODE_URL"

echo "🚀 Deploying WASM contract '${WASM_BIN}' on chain '${CHAIN_ID}' using account '${USER_ADDR}'..."
echo " with cmd : $CMD"
echo "===================================================================="

echo "Storing WASM contract..."
RES=$($CMD tx wasm store "$WASM_BIN" --from "$USER_ADDR" --keyring-backend "test" $TXFLAG -y --output json --keyring-dir "/home/peppi/.neutrond/")
echo "Store transaction result:"
echo "$RES" | jq '.'
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
        echo "$TX_RESULT" | jq '.'
        exit 1
    fi
    echo "Transaction not yet processed. Waiting... (Attempt $((ATTEMPTS+1))/$MAX_ATTEMPTS)"
    sleep 2
    ATTEMPTS=$((ATTEMPTS+1))
done

if [ $ATTEMPTS -eq $MAX_ATTEMPTS ]; then
    echo "Failed to retrieve transaction after $MAX_ATTEMPTS attempts. Last response:"
    echo "$TX_RESULT" | jq '.'
    exit 1
fi

echo "Transaction result:"
echo "$TX_RESULT" | jq '.'

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
echo "INSTANTIATE_MSG: ${INSTANTIATE_MSG}"
echo "--------------------------------------------------------"

# Parse and re-format the INSTANTIATE_MSG
INSTANTIATE_MSG_PARSED=$(echo "$INSTANTIATE_MSG" | jq -r '.')
INSTANTIATE_MSG_ONELINE=$(echo "$INSTANTIATE_MSG_PARSED" | jq '{quartz: .} + {denom: "untrn"}'  )
#jq -c '.'  

echo "Parsed INSTANTIATE_MSG:"
echo "$INSTANTIATE_MSG_PARSED" | jq '.'

INSTANTIATE_CMD="$CMD tx wasm instantiate $CODE_ID '$INSTANTIATE_MSG_ONELINE' --from $USER_ADDR --keyring-backend test --keyring-dir /home/peppi/.neutrond/ --label $LABEL $TXFLAG -y --no-admin --output json"
echo "Instantiate command:"
echo "$INSTANTIATE_CMD"
echo "--------------------------------------------------------"

echo "Executing instantiate command..."
RES=$(eval "$INSTANTIATE_CMD")
echo "Instantiate result:"
echo "$RES" | jq '.'
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
        echo "$TX_RESULT" | jq '.'
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
echo "$RES" | jq '.'
CONTRACT=$(echo "$RES" | jq -r '.contracts[0]')

if [[ -z "$CONTRACT" || "$CONTRACT" == "null" ]]; then
    echo "Failed to retrieve contract address. Printing full query result for debugging:"
    echo "$RES" | jq '.'
    exit 1
fi

echo "🚀 Successfully deployed and instantiated contract!"
echo "🔗 Chain ID: ${CHAIN_ID}"
echo "🆔 Code ID: ${CODE_ID}"
echo "📌 Contract Address: ${CONTRACT}"
echo "🔖 Contract Label: ${LABEL}"