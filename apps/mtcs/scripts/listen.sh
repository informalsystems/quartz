

    ROOT=${ROOT:-$HOME}
    DIR_MTCS="$ROOT/cycles-protocol/quartz-app/"
    DIR_PROTO="$DIR_MTCS/enclave/proto"
    DEFAULT_NODE="127.0.0.1:26657"
    NODE_URL="143.244.186.205:26657"
    # Use the QUARTZ_PORT environment variable if set, otherwise default to 11090
    QUARTZ_PORT="${QUARTZ_PORT:-11090}"


    # Attestation constants
    IAS_API_KEY="669244b3e6364b5888289a11d2a1726d"
    RA_CLIENT_SPID="51CAF5A48B450D624AEFE3286D314894"
    QUOTE_FILE="/tmp/${USER}_test.quote"
    REPORT_FILE="/tmp/${USER}_datareport"
    REPORT_SIG_FILE="/tmp/${USER}_datareportsig"

    if [ "$#" -eq 0 ]; then
        echo "Usage: $0 <contract_address>"
        exit 1  # Exit with a non-zero status to indicate an error
    fi

    CONTRACT=$1

    CMD="wasmd --node http://$NODE_URL"

    WSURL="ws://$NODE_URL/websocket"

    SUBSCRIBE="{\"jsonrpc\":\"2.0\",\"method\":\"subscribe\",\"params\":[\"tm.event='Tx' AND wasm._contract_address = '$CONTRACT' AND wasm.action='init_clearing'\"],\"id\":1}"

    echo $SUBSCRIBE

    echo "--------------------------------------------------------"
    echo "subscribe to events"

    # cat keeps the stdin open so websocat doesnt close
    (echo "$SUBSCRIBE"; cat) | websocat $WSURL | while read msg; do 
        if [[ "$msg" == '{"jsonrpc":"2.0","id":1,"result":{}}' ]]; then
            echo "... subscribed"
            echo "---------------------------------------------------------"
            echo "... waiting for event"
            continue
        fi 

        if echo "$msg" | jq 'has("error")' > /dev/null; then
            echo "... error msg $msg"
            echo "---------------------------------------------------------"
            echo "... waiting for event"
            continue
        fi 


        echo "... received init_clearing event!"
        echo $msg 

        echo "... fetching obligations"

        export EPOCH=$($CMD query wasm contract-state raw "$CONTRACT" "65706f63685f636f756e746572" -o json | jq -r .data | base64 -d)
        PREV_EPOCH=$((EPOCH - 1))

        export OBLIGATIONS=$($CMD query wasm contract-state raw "$CONTRACT" $(printf '%s/%s' "$PREV_EPOCH" "obligations" | hexdump -ve '/1 "%02X"') -o json | jq -r .data | base64 -d)
        export LIQUIDITY_SOURCES=$($CMD query wasm contract-state smart $CONTRACT '{"get_liquidity_sources": {"epoch": '$PREV_EPOCH'}}' -o json | jq -r .data.liquidity_sources)

        COMBINED_JSON=$(jq -nc \
            --argjson intents "$OBLIGATIONS" \
            --argjson liquidity_sources "$LIQUIDITY_SOURCES" \
            '{intents: $intents, liquidity_sources: $liquidity_sources}')

        echo $COMBINED_JSON

        # Wrap the combined JSON string into another JSON object with a "message" field
        REQUEST_MSG=$(jq -nc --arg message "$COMBINED_JSON" '{"message": $message}')

        echo "... executing mtcs"
        export ATTESTED_MSG=$(grpcurl -plaintext -import-path "$DIR_PROTO" -proto mtcs.proto -d "$REQUEST_MSG" "127.0.0.1:$QUARTZ_PORT" mtcs.Clearing/Run | jq -c '.message | fromjson')

        QUOTE=$(echo "$ATTESTED_MSG" | jq -c '.attestation')
        MSG=$(echo "$ATTESTED_MSG" | jq -c '.msg')

        # request the IAS report for EPID attestations
        echo -n "$QUOTE" | xxd -r -p - > "$QUOTE_FILE"
        gramine-sgx-ias-request report -g "$RA_CLIENT_SPID" -k "$IAS_API_KEY" -q "$QUOTE_FILE" -r "$REPORT_FILE" -s "$REPORT_SIG_FILE" > /dev/null 2>&1
        REPORT=$(cat "$REPORT_FILE")
        REPORTSIG=$(cat "$REPORT_SIG_FILE" | tr -d '\r')

        echo "... submitting update"


        export EXECUTE=$(jq -nc --argjson submit_setoffs "$(jq -nc --argjson msg "$MSG" --argjson attestation \
            "$(jq -nc --argjson report "$(jq -nc --argjson report "$REPORT" --arg reportsig "$REPORTSIG" '$ARGS.named')" '$ARGS.named')" \
            '$ARGS.named')" '$ARGS.named')
        $CMD tx wasm execute "$CONTRACT" "$EXECUTE" --from admin --chain-id testing -y --gas 2000000


        echo " ... done"
        echo "---------------------------------------------------------"
        echo "... waiting for event"
    done


