#!/bin/bash

set -eo pipefail

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Function to print colored and formatted messages
print_message() {
    local color=$1
    local message=$2
    echo -e "${color}${BOLD}${message}${NC}"
}

# Function to print section headers
print_header() {
    local message=$1
    echo -e "\n${MAGENTA}${BOLD}======== $message ========${NC}\n"
}

# Function to print success messages
print_success() {
    local message=$1
    echo -e "${GREEN}${BOLD}✅ $message${NC}"
}

# Function to print error messages
print_error() {
    local message=$1
    echo -e "${RED}${BOLD}❌ Error: $message${NC}"
    exit 1
}

# Function to print waiting messages
print_waiting() {
    local message=$1
    echo -e "${YELLOW}${BOLD}⏳ $message${NC}"
}

# Function to update and display progress
update_progress() {
    local step=$1
    local total_steps=$2
    local percentage=$((step * 100 / total_steps))
    print_message $BLUE "Progress: [$percentage%] Step $step of $total_steps"
}


# Set up variables
ROOT=${ROOT:-$(git rev-parse --show-toplevel)}
WASMD_HOME=${WASMD_HOME:-"/home/peppi/.neutrond"}
USER_ADDR=$(neutrond keys show -a "val1" --keyring-backend test --home "$WASMD_HOME" --keyring-dir "/home/peppi/.neutrond/")

if [ -z "$USER_ADDR" ]; then
    print_error "User address not found. Please ensure the key exists in the keyring."
fi

WASM_BIN="$1"
INSTANTIATE_MSG="$2"
CHAIN_ID=${CHAIN_ID:-test-1}
NODE_URL=${NODE_URL:-127.0.0.1:26657}
LABEL=${LABEL:-quartz-transfers-app}
COUNT=${COUNT:-0}
QUARTZ_PORT="${QUARTZ_PORT:-11090}"
TXFLAG="--chain-id ${CHAIN_ID} --gas-prices 0.0025untrn --gas auto --gas-adjustment 1.3"
CMD="neutrond --node http://$NODE_URL"




TOTAL_STEPS=7
CURRENT_STEP=0

update_progress $((++CURRENT_STEP)) $TOTAL_STEPS
print_header "Deploying WASM Contract"
print_message $CYAN "Contract: ${WASM_BIN}"
print_message $CYAN "Chain ID: ${CHAIN_ID}"
print_message $CYAN "User Address: ${USER_ADDR}"
print_message $CYAN "Command: $CMD"

print_message $BLUE "Storing WASM contract..."
RES=$($CMD tx wasm store "$WASM_BIN" --from "$USER_ADDR" --keyring-backend "test" $TXFLAG -y --output json --keyring-dir "/home/peppi/.neutrond/")
TX_HASH=$(echo "$RES" | jq -r '.txhash')
print_message $CYAN "Transaction hash: $TX_HASH"

print_waiting "Waiting for transaction to be included in a block..."
ATTEMPTS=0
MAX_ATTEMPTS=30
while [ $ATTEMPTS -lt $MAX_ATTEMPTS ]; do
    TX_RESULT=$($CMD query tx "$TX_HASH" --output json 2>/dev/null || echo '{"code": 1}')
    TX_CODE=$(echo "$TX_RESULT" | jq -r '.code // .tx_result.code // 1')
    if [[ $TX_CODE == "0" ]]; then
        print_success "Transaction processed successfully."
        break
    elif [[ $TX_CODE != "1" ]]; then
        print_error "Error processing transaction. Code: $TX_CODE"
    fi
    print_waiting "Transaction not yet processed. Waiting... (Attempt $((ATTEMPTS+1))/$MAX_ATTEMPTS)"
    sleep 2
    ATTEMPTS=$((ATTEMPTS+1))
done

if [ $ATTEMPTS -eq $MAX_ATTEMPTS ]; then
    print_error "Failed to retrieve transaction after $MAX_ATTEMPTS attempts."
fi

print_message $BLUE "Extracting CODE_ID..."
CODE_ID=$(echo "$TX_RESULT" | jq -r '.events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')
print_message $CYAN "Extracted CODE_ID: $CODE_ID"

if [[ -z "$CODE_ID" || "$CODE_ID" == "null" ]]; then
    print_error "Failed to extract CODE_ID."
fi

update_progress $((++CURRENT_STEP)) $TOTAL_STEPS
print_header "Instantiating Contract"
print_message $CYAN "Label: ${LABEL}"
print_message $CYAN "Code ID: ${CODE_ID}"

INSTANTIATE_MSG_PARSED=$(echo "$INSTANTIATE_MSG" | jq -r '.')
INSTANTIATE_MSG_ONELINE=$(echo "$INSTANTIATE_MSG_PARSED" | jq '{quartz: .} + {denom: "untrn"}'  )

INSTANTIATE_CMD="$CMD tx wasm instantiate $CODE_ID '$INSTANTIATE_MSG_ONELINE' --from $USER_ADDR --keyring-backend test --keyring-dir /home/peppi/.neutrond/ --label $LABEL $TXFLAG -y --no-admin --output json"

print_message $BLUE "Executing instantiate command..."
RES=$(eval "$INSTANTIATE_CMD")
TX_HASH=$(echo "$RES" | jq -r '.txhash')

print_waiting "Waiting for instantiate transaction to be processed..."
ATTEMPTS=0
while [ $ATTEMPTS -lt $MAX_ATTEMPTS ]; do
    TX_RESULT=$($CMD query tx "$TX_HASH" --output json 2>/dev/null || echo '{"code": 1}')
    TX_CODE=$(echo "$TX_RESULT" | jq -r '.code // .tx_result.code // 1')
    if [[ $TX_CODE == "0" ]]; then
        print_success "Instantiate transaction processed successfully."
        break
    elif [[ $TX_CODE != "1" ]]; then
        print_error "Error processing instantiate transaction. Code: $TX_CODE"
    fi
    print_waiting "Instantiate transaction not yet processed. Waiting... (Attempt $((ATTEMPTS+1))/$MAX_ATTEMPTS)"
    sleep 2
    ATTEMPTS=$((ATTEMPTS+1))
done

if [ $ATTEMPTS -eq $MAX_ATTEMPTS ]; then
    print_error "Failed to retrieve instantiate transaction after $MAX_ATTEMPTS attempts."
fi

print_message $BLUE "Querying for instantiated contract..."
RES=$($CMD query wasm list-contract-by-code "$CODE_ID" --output json)
CONTRACT=$(echo "$RES" | jq -r '.contracts[0]')

if [[ -z "$CONTRACT" || "$CONTRACT" == "null" ]]; then
    print_error "Failed to retrieve contract address."
fi

print_message $CYAN "CONTRACT: $CONTRACT"

cd $ROOT/relayer

update_progress $((++CURRENT_STEP)) $TOTAL_STEPS
print_header "Executing SessionCreate on Enclave"
export EXECUTE_CREATE=$(QUARTZ_PORT=$QUARTZ_PORT ./scripts/relayNeutron.sh SessionCreate)
if [ -z "$EXECUTE_CREATE" ]; then
    print_error "Failed to execute SessionCreate on enclave"
fi
print_success "SessionCreate execution successful"

print_message $BLUE "Submitting SessionCreate to contract..."
RES=$($CMD tx wasm execute "$CONTRACT" "$EXECUTE_CREATE" --from "$USER_ADDR"  $TXFLAG --keyring-backend test --keyring-dir "/home/peppi/.neutrond/" --chain-id test-1 -y --output json)
TX_HASH=$(echo "$RES" | jq -r '.txhash')
if [ -z "$TX_HASH" ] || [ "$TX_HASH" == "null" ]; then
    print_error "Failed to retrieve transaction hash"
fi
print_message $CYAN "Transaction hash: $TX_HASH"

print_waiting "Waiting for transaction to be included in a block..."
ATTEMPTS=0
MAX_ATTEMPTS=30
while [ $ATTEMPTS -lt $MAX_ATTEMPTS ]; do
    TX_RESULT=$($CMD query tx "$TX_HASH" --output json 2>/dev/null || echo '{"code": 1}')
    TX_CODE=$(echo "$TX_RESULT" | jq -r '.code // .tx_result.code // 1')
    if [[ $TX_CODE == "0" ]]; then
        print_success "Transaction processed successfully."
        break
    elif [[ $TX_CODE != "1" ]]; then
        print_error "Error processing transaction. Code: $TX_CODE"
    fi
    print_waiting "Transaction not yet processed. Waiting... (Attempt $((ATTEMPTS+1))/$MAX_ATTEMPTS)"
    sleep 2
    ATTEMPTS=$((ATTEMPTS+1))
done

if [ $ATTEMPTS -eq $MAX_ATTEMPTS ]; then
    print_error "Failed to retrieve transaction after $MAX_ATTEMPTS attempts."
fi


print_success "Handshake process completed"

update_progress $((++CURRENT_STEP)) $TOTAL_STEPS
print_header "Setting Session PK"

cd $ROOT/utils/tm-prover
export PROOF_FILE="light-client-proof.json"
rm -f "$PROOF_FILE"

print_message $BLUE "Removed old $PROOF_FILE"

# print_waiting "Waiting for new blocks to be produced..."
print_waiting "Waiting for transaction to be included in a block..."
ATTEMPTS=0
MAX_ATTEMPTS=30
while [ $ATTEMPTS -lt $MAX_ATTEMPTS ]; do
    TX_RESULT=$($CMD query tx "$TX_HASH" --output json 2>/dev/null || echo '{"code": 1}')
    TX_CODE=$(echo "$TX_RESULT" | jq -r '.code // .tx_result.code // 1')
    if [[ $TX_CODE == "0" ]]; then
        print_success "Transaction processed successfully."
        break
    elif [[ $TX_CODE != "1" ]]; then
        print_error "Error processing transaction. Code: $TX_CODE"
    fi
    print_waiting "Transaction not yet processed. Waiting... (Attempt $((ATTEMPTS+1))/$MAX_ATTEMPTS)"
    sleep 2
    ATTEMPTS=$((ATTEMPTS+1))
done


print_success "Required blocks produced. Proceeding with tm-prover..."

cd "$ROOT/apps/transfers"
export TRUSTED_HASH=$(cat trusted.hash)
export TRUSTED_HEIGHT=$(cat trusted.height)

print_message $CYAN "Trusted hash: $TRUSTED_HASH"
print_message $CYAN "Trusted height: $TRUSTED_HEIGHT"

cd $ROOT/utils/tm-prover
export QUARTZ_SESSION=$($CMD query wasm contract-state raw $CONTRACT $(echo -n "quartz_session" | xxd -p -c 20) --node "http://$NODE_URL")
print_message $CYAN "Quartz Session before prover: $QUARTZ_SESSION"

export PROOF_FILE="light-client-proof.json"
if [ -f "$PROOF_FILE" ]; then
    rm "$PROOF_FILE"
    print_message $BLUE "Removed old $PROOF_FILE"
fi

print_message $BLUE "Running prover to get light client proof..."
cargo run -- --chain-id test-1 \
    --primary "http://$NODE_URL" \
    --witnesses "http://$NODE_URL" \
    --trusted-height $TRUSTED_HEIGHT \
    --trusted-hash $TRUSTED_HASH \
    --contract-address $CONTRACT \
    --storage-key "quartz_session" \
    --trace-file $PROOF_FILE > /dev/null 2>&1

if [ $? -eq 0 ]; then
    print_success "Light client proof generated successfully"
else
    print_error "Failed to generate light client proof"
fi

export POP=$(cat $PROOF_FILE)
export POP_MSG=$(jq -nc --arg message "$POP" '$ARGS.named')

update_progress $((++CURRENT_STEP)) $TOTAL_STEPS

print_header "Executing SessionSetPubKey on Enclave"
cd $ROOT/relayer
export EXECUTE_SETPUB=$(QUARTZ_PORT=$QUARTZ_PORT ./scripts/relayNeutron.sh SessionSetPubKey "$POP_MSG")

RES=$($CMD tx wasm execute "$CONTRACT" "$EXECUTE_SETPUB" --from "$USER_ADDR"  $TXFLAG --keyring-backend test --keyring-dir "/home/peppi/.neutrond/" --chain-id test-1 -y --output json)
TX_HASH=$(echo $RES | jq -r '.txhash')

if [ -z "$TX_HASH" ] || [ "$TX_HASH" == "null" ]; then
    print_error "Failed to retrieve transaction hash"
fi
print_message $CYAN "Transaction hash: $TX_HASH"

print_waiting "Waiting for transaction to commit..."
ATTEMPTS=0
MAX_ATTEMPTS=30
while [ $ATTEMPTS -lt $MAX_ATTEMPTS ]; do
    if $CMD query tx "$TX_HASH" &> /dev/null; then
        print_success "Transaction committed successfully"
        break
    fi
    print_waiting "Waiting for tx (Attempt $((ATTEMPTS+1))/$MAX_ATTEMPTS)"
    sleep 2
    ATTEMPTS=$((ATTEMPTS+1))
done

if [ $ATTEMPTS -eq $MAX_ATTEMPTS ]; then
    print_error "Transaction failed to commit after $MAX_ATTEMPTS attempts"
fi

update_progress $((++CURRENT_STEP)) $TOTAL_STEPS
print_header "Checking Session Success"
export NONCE_AND_KEY=$($CMD query wasm contract-state raw "$CONTRACT" $(printf '%s' "quartz_session" | hexdump -ve '/1 "%02X"') -o json | jq -r .data | base64 -d)
# echo $NONCE_AND_KEY
export PUBKEY=$(echo $NONCE_AND_KEY  | jq -r .pub_key)

update_progress $TOTAL_STEPS $TOTAL_STEPS
print_header "Deployment Summary"
print_success "Deployment and handshake completed successfully!"
echo -e "${CYAN}${BOLD}"
echo "📌 Contract Details:"
echo "   • Address: ${CONTRACT}"
echo "   • Code ID: ${CODE_ID}"
echo "   • Label: ${LABEL}"
echo "   • Chain ID: ${CHAIN_ID}"
echo
echo "🔑 Contract Key Information:"
echo "   • Public Key: ${PUBKEY}"
echo
echo "🌐 Network Information:"
echo "   • Node URL: ${NODE_URL}"
echo "   • Quartz Port: ${QUARTZ_PORT}"
echo
echo "👤 User Information:"
echo "   • Address: ${USER_ADDR}"
echo "   • Keyring Backend: test"
echo "   • Keyring Directory: /home/peppi/.neutrond/"
echo
echo "🔧 Additional Settings:"
echo "   • Gas Prices: 0.0025untrn"
echo "   • Gas Adjustment: 1.3"
echo -e "${NC}"

# ASCII art logo for QUARTZ
cat << "EOF"

  ██████  ██    ██  █████  ██████  ████████ ███████ 
 ██    ██ ██    ██ ██   ██ ██   ██    ██         ██   
 ██    ██ ██    ██ ███████ ██████     ██    ███████ 
 ██ ▄▄ ██ ██    ██ ██   ██ ██   ██    ██    ██      
  ██████   ██████  ██   ██ ██   ██    ██    ███████ 
     ▀▀                                             
                        POWERED BY INFORMAL.SYSTEMMS
EOF

print_message $MAGENTA "We hope you'll enjoy developing Quartz Apps!"