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
    echo -e "${RED}${BOLD}❌ Error: $message${NC}" >&2
}



# Configuration
DEFAULT_NODE="127.0.0.1:26657"
NODE_URL=${NODE_URL:-$DEFAULT_NODE}
QUARTZ_PORT="${QUARTZ_PORT:-11090}"

if [ "$#" -eq 0 ]; then
    echo "Usage: $0 <contract_address>"
    exit 1
fi

CONTRACT=$1
CMD="neutrond --node http://$NODE_URL"
WSURL="ws://$NODE_URL/websocket"
ROOT=${ROOT:-$(git rev-parse --show-toplevel)}

WASMD_HOME=${WASMD_HOME:-"$HOME/.neutrond"}
CHAIN_ID=${CHAIN_ID:-test-1}
TXFLAG="--chain-id ${CHAIN_ID} --gas-prices 0.0025untrn --gas auto --gas-adjustment 1.3"
USER_ADDR=$(neutrond keys show -a "val1" --keyring-backend test --home "$WASMD_HOME" --keyring-dir "$WASMD_HOME")

# Subscription queries
SUBSCRIBE_TRANSFER="{\"jsonrpc\":\"2.0\",\"method\": \"subscribe\" ,\"params\":{\"query\":\"execute._contract_address = '$CONTRACT' AND wasm-transfer.action = 'user'\"},\"id\":2}"
SUBSCRIBE_QUERY="{\"jsonrpc\":\"2.0\",\"method\": \"subscribe\" ,\"params\":{\"query\":\"execute._contract_address = '$CONTRACT' AND wasm-query_balance.query  = 'user'\"},\"id\":3}"

# Attestation constants
IAS_API_KEY="669244b3e6364b5888289a11d2a1726d"
RA_CLIENT_SPID="51CAF5A48B450D624AEFE3286D314894"
QUOTE_FILE="/tmp/${USER}_test.quote"
REPORT_FILE="/tmp/${USER}_datareport"
REPORT_SIG_FILE="/tmp/${USER}_datareportsig"

process_json() {
    echo "Raw message: $msg" >&2
    local json_input="$1"
    local result
    result=$(echo "$json_input" | jq -r '.result // empty' 2>&1) || {
        echo "Error parsing JSON: $result" >&2
        echo "No relevant wasm events found"
        return
    }
    echo "$result"
   
}

wait_for_next_block() {
    local current_height=$($CMD status | jq -r .sync_info.latest_block_height)
    local next_height=$((current_height + 1))
    while [ "$($CMD status 2>&1 | jq -r .sync_info.latest_block_height)" -lt "$next_height" ]; do
        echo "Waiting for next block..."
        sleep 1
    done
}

handle_wasm_transfer() {
    print_header "Received wasm-transfer event"
    echo "Debug: Entering handle_wasm_transfer function"

    wait_for_next_block

    echo "Fetching requests and state..."
    REQUESTS=$($CMD query wasm contract-state raw $CONTRACT $(printf '%s' "requests" | hexdump -ve '/1 "%02X"') -o json | jq -r .data | base64 -d)
    STATE=$($CMD query wasm contract-state raw $CONTRACT $(printf '%s' "state" | hexdump -ve '/1 "%02X"') -o json | jq -r .data | base64 -d)

    cd "$ROOT/apps/transfers"
    export TRUSTED_HASH=$(cat trusted.hash)
    export TRUSTED_HEIGHT=$(cat trusted.height)

    cd $ROOT/utils/tm-prover
    export PROOF_FILE="light-client-proof.json"
    [ -f "$PROOF_FILE" ] && rm "$PROOF_FILE" && echo "Removed old $PROOF_FILE"

    echo "Trusted hash: $TRUSTED_HASH"
    echo "Trusted height: $TRUSTED_HEIGHT"
    echo "Contract: $CONTRACT"

    echo "Running prover to get light client proof..."
    cargo run -- --chain-id $CHAIN_ID \
        --primary "http://$NODE_URL" \
        --witnesses "http://$NODE_URL" \
        --trusted-height $TRUSTED_HEIGHT \
        --trusted-hash $TRUSTED_HASH \
        --contract-address $CONTRACT \
        --storage-key "requests" \
        --trace-file $PROOF_FILE

    export POP=$(cat $PROOF_FILE)
    export ENCLAVE_REQUEST=$(jq -nc --argjson requests "$REQUESTS" --argjson state $STATE '$ARGS.named')
    export REQUEST_MSG=$(jq --argjson msg "$ENCLAVE_REQUEST" '. + {msg: $msg}' <<< "$POP")
    export PROTO_MSG=$(jq -nc --arg message "$REQUEST_MSG" '$ARGS.named')

    cd $ROOT/apps/transfers/enclave

    echo "Executing transfer..."
    ATTESTED_MSG=$(grpcurl -plaintext -import-path ./proto/ -proto transfers.proto \
        -d "$PROTO_MSG" "127.0.0.1:$QUARTZ_PORT" transfers.Settlement/Run | \
        jq .message | jq -R 'fromjson | fromjson' | jq -c)
    QUOTE=$(echo "$ATTESTED_MSG" | jq -c '.attestation')
    MSG=$(echo "$ATTESTED_MSG" | jq -c '.msg')



    if [ -n "$MOCK_SGX" ]; then
        echo "Running in MOCK_SGX mode"
        EXECUTE=$(jq -nc --argjson update "$(jq -nc --argjson msg "$MSG" --argjson attestation "$QUOTE" '$ARGS.named')" '$ARGS.named')
    else
        echo "Getting report..."
        echo -n "$QUOTE" | xxd -r -p - > "$QUOTE_FILE"
        gramine-sgx-ias-request report -g "$RA_CLIENT_SPID" -k "$IAS_API_KEY" -q "$QUOTE_FILE" \
            -r "$REPORT_FILE" -s "$REPORT_SIG_FILE" > /dev/null 2>&1
        REPORT=$(cat "$REPORT_FILE")
        REPORTSIG=$(cat "$REPORT_SIG_FILE" | tr -d '\r')
        EXECUTE=$(jq -nc --argjson update "$(jq -nc --argjson msg "$MSG" --argjson attestation \
            "$(jq -nc --argjson report "$(jq -nc --argjson report "$REPORT" \
            --arg reportsig "$REPORTSIG" '$ARGS.named')" '$ARGS.named')" '$ARGS.named')" '$ARGS.named')
    fi

    echo "Submitting update..."
    echo $EXECUTE | jq '.'
    $CMD tx wasm execute "$CONTRACT" "$EXECUTE" --from "$USER_ADDR" $TXFLAG -y 
    print_success "Transfer executed"
}

handle_wasm_query_balance() {
    print_header "Received wasm-query_balance event"
    echo "Debug: Entering handle_wasm_query_balance function"

    echo "Fetching state..."

    STATE=$($CMD query wasm contract-state raw $CONTRACT $(printf '%s' "state" | hexdump -ve '/1 "%02X"') -o json | jq -r .data | base64 -d)
    ADDRESS=$(echo "$1" | jq -r '.result.events["message.sender"][0]')
    EPHEMERAL_PUBKEY=$(echo "$1" | jq -r '.result.events["wasm-query_balance.emphemeral_pubkey"][0]')

    export ENCLAVE_REQUEST=$(jq -nc --argjson state "$STATE" --arg address "$ADDRESS" --arg ephemeral_pubkey "$EPHEMERAL_PUBKEY" '$ARGS.named')
    export REQUEST_MSG=$(jq -nc --arg message "$ENCLAVE_REQUEST" '$ARGS.named')

    cd $ROOT/apps/transfers/enclave

    echo "Executing query balance..."
    ATTESTED_MSG=$(grpcurl -plaintext -import-path ./proto/ -proto transfers.proto \
        -d "$REQUEST_MSG" "127.0.0.1:$QUARTZ_PORT" transfers.Settlement/Query | jq -r '.message | fromjson')
    QUOTE=$(echo "$ATTESTED_MSG" | jq -c '.attestation')
    MSG=$(echo "$ATTESTED_MSG" | jq -c '.msg')
    QUERY_RESPONSE_MSG=$(jq -n --arg address "$ADDRESS" --argjson msg "$MSG" '{address: $address, encrypted_bal: $msg.encrypted_bal}')

    if [ -n "$MOCK_SGX" ]; then
        echo "Running in MOCK_SGX mode"
        EXECUTE=$(jq -nc --argjson query_response "$(jq -nc --argjson msg "$QUERY_RESPONSE_MSG" --argjson attestation "$QUOTE" '$ARGS.named')" '{query_response: $query_response}')
    else
        echo -n "$QUOTE" | xxd -r -p - > "$QUOTE_FILE"
        gramine-sgx-ias-request report -g "$RA_CLIENT_SPID" -k "$IAS_API_KEY" -q "$QUOTE_FILE" \
            -r "$REPORT_FILE" -s "$REPORT_SIG_FILE" > /dev/null 2>&1
        REPORT=$(cat "$REPORT_FILE")
        REPORTSIG=$(cat "$REPORT_SIG_FILE" | tr -d '\r')
        EXECUTE=$(jq -nc --argjson query_response "$(jq -nc --argjson msg "$QUERY_RESPONSE_MSG" \
            --argjson attestation "$(jq -nc --argjson report "$(jq -nc --argjson report "$REPORT" \
            --arg reportsig "$REPORTSIG" '$ARGS.named')" '$ARGS.named')" '$ARGS.named')" \
            '{query_response: $query_response}')
    fi

    echo "Submitting update..."
    echo $EXECUTE | jq '.'
    $CMD tx wasm execute "$CONTRACT" "$EXECUTE" --from "$USER_ADDR" $TXFLAG -y 
    print_success "Query balance executed"
}


# Main loop
( echo "$SUBSCRIBE_TRANSFER"; echo "$SUBSCRIBE_QUERY"; cat) | websocat $WSURL | while read -r msg; do 
    EVENTS=$(process_json "$msg")

    if [[ "$EVENTS" == "JSON parsing failed" ]]; then
        print_error "Failed to parse JSON message. Skipping this message."
        continue
    fi

    if [[ -z "$EVENTS" || "$EVENTS" == "No relevant wasm events found" ]]; then
        if [[ "$msg" == *'"result":{}'* ]]; then
            print_success "Subscribed to $msg"
            print_message "$YELLOW" "Waiting for event..."
        else
            print_message "$YELLOW" "No relevant events found in message. Waiting for next event..."
        fi
        continue
    fi

    if echo "$EVENTS" | grep -q 'wasm-transfer'; then
        handle_wasm_transfer
    elif echo "$EVENTS" | grep -q 'wasm-query_balance'; then
        handle_wasm_query_balance "$msg"
    fi

    print_message "$YELLOW" "Waiting for next event..."
done