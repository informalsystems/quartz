#!/bin/bash

ROOT=${ROOT:-$HOME}
DEFAULT_NODE="127.0.0.1:26657"
NODE_URL=${NODE_URL:-$DEFAULT_NODE}
# Use the QUARTZ_PORT environment variable if set, otherwise default to 11090
QUARTZ_PORT="${QUARTZ_PORT:-11090}"

if [ "$#" -eq 0 ]; then
    echo "Usage: $0 <contract_address>"
    exit 1  # Exit with a non-zero status to indicate an error
fi

CONTRACT=$1
CMD="wasmd --node http://$NODE_URL"
WSURL="ws://$NODE_URL/websocket"

SUBSCRIBE_TRANSFER="{\"jsonrpc\":\"2.0\",\"method\":\"subscribe\",\"params\":[\"execute._contract_address = '$CONTRACT' AND wasm-transfer.action = 'user'\"],\"id\":1}"
SUBSCRIBE_QUERY="{\"jsonrpc\":\"2.0\",\"method\":\"subscribe\",\"params\":[\"execute._contract_address = '$CONTRACT' AND wasm-query_balance.query = 'user'\"],\"id\":2}"
# Attestation constants
IAS_API_KEY="669244b3e6364b5888289a11d2a1726d"
RA_CLIENT_SPID="51CAF5A48B450D624AEFE3286D314894"
QUOTE_FILE="/tmp/${USER}_test.quote"
REPORT_FILE="/tmp/${USER}_datareport"
REPORT_SIG_FILE="/tmp/${USER}_datareportsig"

# cat keeps the stdin open so websocat doesnt close
(echo "$SUBSCRIBE_TRANSFER"; echo "$SUBSCRIBE_QUERY"; cat) | websocat $WSURL | while read msg; do 
    if [[ "$msg" == '{"jsonrpc":"2.0","id":1,"result":{}}' ]] || \
       [[ "$msg" == '{"jsonrpc":"2.0","id":2,"result":{}}' ]]; then
        echo "---------------------------------------------------------"
        echo "... subscribed to $msg"
        echo "... waiting for event"
        continue
    fi 

    CLEAN_MSG=$(echo "$msg" | sed 's/"log":"\[.*\]"/"log":"<invalid_json>"/' | jq '.result.events')

    if echo "$CLEAN_MSG" | grep -q 'wasm-transfer'; then
        echo "---------------------------------------------------------"
        echo "... received wasm-transfer event!"

        current_height=$($CMD status | jq -r .SyncInfo.latest_block_height)
        next_height=$((current_height + 1))

        while [ "$($CMD status 2>&1 | jq -r .SyncInfo.latest_block_height)" -lt "$next_height" ]; do
            echo "waiting for next block"
            sleep 1
        done

        echo "... fetching requests"
        REQUESTS=$($CMD query wasm contract-state raw $CONTRACT $(printf '%s' "requests" | \
            hexdump -ve '/1 "%02X"') -o json | jq -r .data | base64 -d)
        STATE=$($CMD query wasm contract-state raw $CONTRACT $(printf '%s' "state" | \
            hexdump -ve '/1 "%02X"') -o json | jq -r .data | base64 -d)

        cd "$ROOT/cycles-quartz/apps/transfers"
        export TRUSTED_HASH=$(cat trusted.hash)
        export TRUSTED_HEIGHT=$(cat trusted.height)

        cd $ROOT/cycles-quartz/utils/tm-prover
        export PROOF_FILE="light-client-proof.json"
        if [ -f "$PROOF_FILE" ]; then
            rm "$PROOF_FILE"
            echo "removed old $PROOF_FILE"
        fi

        echo "trusted hash $TRUSTED_HASH"
        echo "trusted hash $TRUSTED_HEIGHT"
        echo "contract $CONTRACT"

        # run prover to get light client proof
        cargo run -- --chain-id testing \
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

        cd $ROOT/cycles-quartz/apps/transfers/enclave

        echo "... executing transfer"
        export ATTESTED_MSG=$(grpcurl -plaintext -import-path ./proto/ -proto transfers.proto \
            -d "$PROTO_MSG" "127.0.0.1:$QUARTZ_PORT" transfers.Settlement/Run | \
            jq .message | jq -R 'fromjson | fromjson' | jq -c)
        QUOTE=$(echo "$ATTESTED_MSG" | jq -c '.attestation')
        MSG=$(echo "$ATTESTED_MSG" | jq -c '.msg')

        if [ -n "$MOCK_SGX" ]; then
            echo "... running in MOCK_SGX mode"
            EXECUTE=$(jq -nc --argjson update "$(jq -nc --argjson msg "$MSG" \
                --argjson attestation "$QUOTE" '$ARGS.named')" '$ARGS.named')
        else
            echo "... getting report"
            echo -n "$QUOTE" | xxd -r -p - > "$QUOTE_FILE"
            gramine-sgx-ias-request report -g "$RA_CLIENT_SPID" -k "$IAS_API_KEY" -q "$QUOTE_FILE" \
                -r "$REPORT_FILE" -s "$REPORT_SIG_FILE" > /dev/null 2>&1
            REPORT=$(cat "$REPORT_FILE")
            REPORTSIG=$(cat "$REPORT_SIG_FILE" | tr -d '\r')

            EXECUTE=$(jq -nc --argjson update "$(jq -nc --argjson msg "$MSG" --argjson attestation \
                "$(jq -nc --argjson report "$(jq -nc --argjson report "$REPORT" \
                --arg reportsig "$REPORTSIG" '$ARGS.named')" '$ARGS.named')" '$ARGS.named')" '$ARGS.named')
        fi

        echo "... submitting update"
        echo $EXECUTE | jq '.'
        $CMD tx wasm execute "$CONTRACT" "$EXECUTE" --from admin --chain-id testing -y --gas 2000000

        echo " ... done"
        echo "---------------------------------------------------------"
        echo "... waiting for event"
    elif echo "$CLEAN_MSG" | grep -q 'wasm-query_balance'; then
        echo "... received wasm-query_balance event!"
        echo "... fetching state"

        STATE=$($CMD query wasm contract-state raw $CONTRACT $(printf '%s' "state" | \
            hexdump -ve '/1 "%02X"') -o json | jq -r .data | base64 -d)

        ADDRESS=$(echo "$msg" | sed 's/"log":"\[.*\]"/"log":"<invalid_json>"/' | \
            jq -r '.result.events["message.sender"]'[0])

        EPHEMERAL_PUBKEY=$(echo "$msg" | sed 's/"log":"\[.*\]"/"log":"<invalid_json>"/' | \
            jq -r '.result.events["wasm-query_balance.emphemeral_pubkey"]'[0])

        export ENCLAVE_REQUEST=$(jq -nc --argjson state "$STATE" --arg address "$ADDRESS" \
            --arg ephemeral_pubkey "$EPHEMERAL_PUBKEY" '$ARGS.named')
        export REQUEST_MSG=$(jq -nc --arg message "$ENCLAVE_REQUEST" '$ARGS.named')

        cd $ROOT/cycles-quartz/apps/transfers/enclave

        echo "... executing query balance"
        ATTESTED_MSG=$(grpcurl -plaintext -import-path ./proto/ -proto transfers.proto \
            -d "$REQUEST_MSG" "127.0.0.1:$QUARTZ_PORT" transfers.Settlement/Query | jq -r '.message | fromjson')
        QUOTE=$(echo "$ATTESTED_MSG" | jq -c '.attestation')
        MSG=$(echo "$ATTESTED_MSG" | jq -c '.msg')
        QUERY_RESPONSE_MSG=$(jq -n --arg address "$ADDRESS" --argjson msg "$MSG" \
            '{address: $address, encrypted_bal: $msg.encrypted_bal}')

        if [ -n "$MOCK_SGX" ]; then
            echo "... running in MOCK_SGX mode"
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

        echo "... submitting update"
        echo $EXECUTE | jq '.'
        $CMD tx wasm execute "$CONTRACT" "$EXECUTE" --from admin --chain-id testing -y --gas 2000000
        echo " ... done"
        echo "------------------------------------"
        echo "... waiting for event"
    fi
done