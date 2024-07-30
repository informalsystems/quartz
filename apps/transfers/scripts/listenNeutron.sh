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

# Set up variables
ROOT=${ROOT:-$(git rev-parse --show-toplevel)}
NODE_URL=${NODE_URL:-127.0.0.1:26657}
QUARTZ_PORT="${QUARTZ_PORT:-11090}"

if [ "$#" -eq 0 ]; then
    print_error "Usage: $0 <contract_address>"
    exit 1
fi

CONTRACT=$1
CMD="neutrond --node http://$NODE_URL"
WSURL="ws://$NODE_URL/websocket"

# Subscription messages
# SUBSCRIBE_TRANSFER=$(cat <<EOF
# {
#   "jsonrpc": "2.0",
#   "method": "subscribe",
#   "id": 1,
#   "params": {
#     "query": "tm.event='Tx' AND execute._contract_address='$CONTRACT' AND wasm-transfer.action='user'"
#   }
# }
# EOF
# )

# SUBSCRIBE_QUERY=$(cat <<EOF
# {
#   "jsonrpc": "2.0",
#   "method": "subscribe",
#   "id": 2,
#   "params": {
#     "query": "tm.event='Tx' AND execute._contract_address='$CONTRACT' AND wasm-store_balance.query='user'"
#   }
# }
# EOF
# )

SUBSCRIBE_TRANSFER="{\"jsonrpc\":\"2.0\",\"method\":\"subscribe\",\"params\":[\"execute._contract_address = '$CONTRACT' AND wasm-transfer.action = 'user'\"],\"id\":1}"
SUBSCRIBE_QUERY="{\"jsonrpc\":\"2.0\",\"method\":\"subscribe\",\"params\":[\"execute._contract_address = '$CONTRACT' AND wasm-query_balance.query = 'user'\"],\"id\":2}"



# Attestation constants
IAS_API_KEY="669244b3e6364b5888289a11d2a1726d"
RA_CLIENT_SPID="51CAF5A48B450D624AEFE3286D314894"
QUOTE_FILE="/tmp/${USER}_test.quote"
REPORT_FILE="/tmp/${USER}_datareport"
REPORT_SIG_FILE="/tmp/${USER}_datareportsig"

print_header "Starting WebSocket Listener"

send_subscription() {
    local subscription="$1"
    echo "$subscription" | websocat --text "$WSURL"
    read -r response
    handle_message "$response"
}

handle_message() {
    local msg="$1"
    if jq -e '.result' <<< "$msg" > /dev/null 2>&1; then
        if jq -e '.result.query' <<< "$msg" > /dev/null 2>&1; then
            print_success "Subscribed successfully"
            return 0
        fi
        CLEAN_MSG=$(jq -r '.result.events // empty' <<< "$msg")
        if [[ -n "$CLEAN_MSG" ]]; then
            if echo "$CLEAN_MSG" | grep -q 'wasm-transfer'; then
                handle_transfer_event
            elif echo "$CLEAN_MSG" | grep -q 'wasm-query_balance'; then
                handle_query_balance_event "$msg"
            fi
        fi
    elif jq -e '.error' <<< "$msg" > /dev/null 2>&1; then
        print_error "Error in message: $msg"
        return 1
    else
        print_error "Unexpected message format: $msg"
        return 1
    fi
}

handle_transfer_event() {
    print_header "Handling Transfer Event"

    current_height=$($CMD status | jq -r .sync_info.latest_block_height)
    next_height=$((current_height + 1))

    while [ "$($CMD status 2>&1 | jq -r .sync_info.latest_block_height)" -lt "$next_height" ]; do
        sleep 1
    done

    REQUESTS=$($CMD query wasm contract-state raw $CONTRACT $(printf '%s' "requests" | \
        hexdump -ve '/1 "%02X"') -o json | jq -r .data | base64 -d)
    STATE=$($CMD query wasm contract-state raw $CONTRACT $(printf '%s' "state" | \
        hexdump -ve '/1 "%02X"') -o json | jq -r .data | base64 -d)

    cd "$ROOT/apps/transfers"
    export TRUSTED_HASH=$(cat trusted.hash)
    export TRUSTED_HEIGHT=$(cat trusted.height)

    cd $ROOT/utils/tm-prover
    export PROOF_FILE="light-client-proof.json"
    [ -f "$PROOF_FILE" ] && rm "$PROOF_FILE"

    cargo run -- --chain-id test-1 \
        --primary "$NODE_URL" \
        --witnesses "$NODE_URL" \
        --trusted-height $TRUSTED_HEIGHT \
        --trusted-hash $TRUSTED_HASH \
        --contract-address $CONTRACT \
        --storage-key "requests" \
        --trace-file $PROOF_FILE > /dev/null 2>&1

    export POP=$(cat $PROOF_FILE)

    export ENCLAVE_REQUEST=$(jq -nc --argjson requests "$REQUESTS" --argjson state $STATE '$ARGS.named')
    export REQUEST_MSG=$(jq --argjson msg "$ENCLAVE_REQUEST" '. + {msg: $msg}' <<< "$POP")
    export PROTO_MSG=$(jq -nc --arg message "$REQUEST_MSG" '$ARGS.named')

    cd $ROOT/apps/transfers/enclave

    export ATTESTED_MSG=$(grpcurl -plaintext -import-path ./proto/ -proto transfers.proto \
        -d "$PROTO_MSG" "127.0.0.1:$QUARTZ_PORT" transfers.Settlement/Run | \
        jq .message | jq -R 'fromjson | fromjson' | jq -c)
    QUOTE=$(echo "$ATTESTED_MSG" | jq -c '.attestation')
    MSG=$(echo "$ATTESTED_MSG" | jq -c '.msg')

    if [ -n "$MOCK_SGX" ]; then
        EXECUTE=$(jq -nc --argjson update "$(jq -nc --argjson msg "$MSG" \
            --argjson attestation "$QUOTE" '$ARGS.named')" '$ARGS.named')
    else
        echo -n "$QUOTE" | xxd -r -p - > "$QUOTE_FILE"
        gramine-sgx-ias-request report -g "$RA_CLIENT_SPID" -k "$IAS_API_KEY" -q "$QUOTE_FILE" \
            -r "$REPORT_FILE" -s "$REPORT_SIG_FILE" > /dev/null 2>&1
        REPORT=$(cat "$REPORT_FILE")
        REPORTSIG=$(cat "$REPORT_SIG_FILE" | tr -d '\r')

        EXECUTE=$(jq -nc --argjson update "$(jq -nc --argjson msg "$MSG" --argjson attestation \
            "$(jq -nc --argjson report "$(jq -nc --argjson report "$REPORT" \
            --arg reportsig "$REPORTSIG" '$ARGS.named')" '$ARGS.named')" '$ARGS.named')" '$ARGS.named')
    fi

    $CMD tx wasm execute "$CONTRACT" "$EXECUTE" --from "$USER_ADDR" $TXFLAG --keyring-backend test --keyring-dir "/home/peppi/.neutrond/" --chain-id test-1 -y --output json > /dev/null

    print_success "Transfer execution completed"
}

handle_query_balance_event() {
    local msg="$1"
    print_header "Handling Query Balance Event"

    STATE=$($CMD query wasm contract-state raw $CONTRACT $(printf '%s' "state" | \
        hexdump -ve '/1 "%02X"') -o json | jq -r .data | base64 -d)

    ADDRESS=$(echo "$msg" | sed 's/"log":"\[.*\]"/"log":"<invalid_json>"/' | \
        jq -r '.result.events["message.sender"]'[0])

    EPHEMERAL_PUBKEY=$(echo "$msg" | sed 's/"log":"\[.*\]"/"log":"<invalid_json>"/' | \
        jq -r '.result.events["wasm-store_balance.emphemeral_pubkey"]'[0])

    export ENCLAVE_REQUEST=$(jq -nc --argjson state "$STATE" --arg address "$ADDRESS" \
        --arg ephemeral_pubkey "$EPHEMERAL_PUBKEY" '$ARGS.named')
    export REQUEST_MSG=$(jq -nc --arg message "$ENCLAVE_REQUEST" '$ARGS.named')

    cd $ROOT/apps/transfers/enclave

    ATTESTED_MSG=$(grpcurl -plaintext -import-path ./proto/ -proto transfers.proto \
        -d "$REQUEST_MSG" "127.0.0.1:$QUARTZ_PORT" transfers.Settlement/Query | jq -r '.message | fromjson')
    QUOTE=$(echo "$ATTESTED_MSG" | jq -c '.attestation')
    MSG=$(echo "$ATTESTED_MSG" | jq -c '.msg')
    QUERY_RESPONSE_MSG=$(jq -n --arg address "$ADDRESS" --argjson msg "$MSG" \
        '{address: $address, encrypted_bal: $msg.encrypted_bal}')

    if [ -n "$MOCK_SGX" ]; then
        EXECUTE=$(jq -nc --argjson query_response "$(jq -nc --argjson msg "$QUERY_RESPONSE_MSG" \
            --argjson attestation "$QUOTE" '$ARGS.named')" '{query_response: $query_response}')
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

    $CMD tx wasm execute "$CONTRACT" "$EXECUTE" --from "$USER_ADDR" $TXFLAG --keyring-backend test --keyring-dir "/home/peppi/.neutrond/" --chain-id test-1 -y --output json > /dev/null

    print_success "Query balance execution completed"
}

connect_websocket() {
    (
        send_subscription "$SUBSCRIBE_TRANSFER"
        send_subscription "$SUBSCRIBE_QUERY"
        while true; do
            read -r msg
            if ! handle_message "$msg"; then
                print_error "Error handling message, reconnecting..."
                return 1
            fi
        done
    ) | websocat --text "$WSURL"
}

while true; do
    if ! connect_websocket; then
        print_error "WebSocket connection failed, retrying in 5 seconds..."
        sleep 5
    fi
done