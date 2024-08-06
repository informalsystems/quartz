#!/bin/bash
#
# Perform the SessionCreate and SessionSetPubKey handshake between the contract and the sgx node
# Expects:
#   - enclave is already initialized
#   - contract is already deployed
#   - apps/transfers/trusted.hash exists
#

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


ROOT=${ROOT:-$(git rev-parse --show-toplevel)}

NODE_URL=${NODE_URL:-127.0.0.1:26657}
TXFLAG="--chain-id test-1 --gas-prices 0.0025untrn --gas auto --gas-adjustment 1.3"

if [ "$#" -eq 0 ]; then
    echo "Usage: $0 <contract_address>"
    exit 1  # Exit with a non-zero status to indicate an error
fi

CONTRACT="$1" 

WASMD_ROOT=${WASMD_ROOT:-"$HOME/.neutrond"}

USER_ADDR=$(neutrond keys show -a "val1" --keyring-backend "test" --home "$WASMD_ROOT" --keyring-dir "$WASMD_ROOT")


CMD="neutrond --node http://$NODE_URL"

cd "$ROOT/apps/transfers"
export TRUSTED_HASH=$(cat trusted.hash)
export TRUSTED_HEIGHT=$(cat trusted.height)

echo "using CMD: $CMD"
echo "--------------------------------------------------------"

echo "create session"

# change to relay dir
cd $ROOT/relayer

# execute SessionCreate on enclave
export EXECUTE_CREATE=$(./scripts/relay.sh SessionCreate)
echo $EXECUTE_CREATE

# submit SessionCreate to contract
RES=$($CMD tx wasm execute "$CONTRACT" "$EXECUTE_CREATE" --from "$USER_ADDR" $TXFLAG --keyring-backend "test" --home "$WASMD_ROOT" --keyring-dir "$WASMD_ROOT"  -y --output json)
TX_HASH=$(echo $RES | jq -r '.txhash')

# # wait for tx to commit
# while ! $CMD query tx $TX_HASH &> /dev/null; do
#     echo "... 🕐 waiting for tx $TX_HASH"
#     sleep 1 
# done 

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




# need to wait another block for light client proof
BLOCK_HEIGHT=$($CMD  status | jq -r .sync_info.latest_block_height)
echo "at height $BLOCK_HEIGHT. need to wait for a block"
while [[ $BLOCK_HEIGHT == $($CMD query block --type=height $BLOCK_HEIGHT | jq -r .block.header.height) ]]; do
    echo "... 🕐 waiting for another block"
    sleep 1
done

echo "--------------------------------------------------------"

echo "set session pk"

# change to prover dir
cd $ROOT/utils/tm-prover
export PROOF_FILE="light-client-proof.json"
if [ -f "$PROOF_FILE" ]; then
    rm "$PROOF_FILE"
    echo "removed old $PROOF_FILE"
fi

# TODO: pass this in?
echo "trusted hash $TRUSTED_HASH"
echo "trusted hash $TRUSTED_HEIGHT"
echo "contract $CONTRACT"

# run prover to get light client proof
# TODO: assume this binary is pre-built?
# TODO: pass in addresses and chain id 
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

# execute SessionSetPubKey on enclave
cd $ROOT/relayer
export EXECUTE_SETPUB=$(./scripts/relay.sh SessionSetPubKey "$POP_MSG")

RES=$($CMD tx wasm execute "$CONTRACT" "$EXECUTE_SETPUB" --from "$USER_ADDR" --keyring-backend "test" --keyring-dir "$WASMD_ROOT" $TXFLAG -y --output json)
TX_HASH=$(echo $RES | jq -r '.txhash')

# wait for tx to commit
while ! $CMD query tx $TX_HASH &> /dev/null; do
    echo "... 🕐 waiting for tx $TX_HASH"
    sleep 1 
done 

echo "--------------------------------------------------------"

# echo "check session success"
export NONCE_AND_KEY=$($CMD query wasm contract-state raw "$CONTRACT" $(printf '%s' "quartz_session" | hexdump -ve '/1 "%02X"') -o json | jq -r .data | base64 -d)
# echo $NONCE_AND_KEY
export PUBKEY=$(echo $NONCE_AND_KEY  | jq -r .pub_key)

