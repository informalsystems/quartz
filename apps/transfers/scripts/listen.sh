ROOT=${ROOT:-$HOME}

DEFAULT_NODE="127.0.0.1:26657"
NODE_URL=${NODE_URL:-$DEFAULT_NODE}

if [ "$#" -eq 0 ]; then
    echo "Usage: $0 <contract_address>"
    exit 1  # Exit with a non-zero status to indicate an error
fi

CONTRACT=$1

CMD="wasmd --node http://$NODE_URL"

WSURL="ws://$NODE_URL/websocket"

SUBSCRIBE_TRANSFER="{\"jsonrpc\":\"2.0\",\"method\":\"subscribe\",\"params\":[\"execute._contract_address = '$CONTRACT' AND wasm-transfer.action = 'user'\"],\"id\":1}"
SUBSCRIBE_QUERY="{\"jsonrpc\":\"2.0\",\"method\":\"subscribe\",\"params\":[\"execute._contract_address = '$CONTRACT' AND wasm-query_balance.query = 'user'\"],\"id\":2}"

echo $SUBSCRIBE_TRANSFER
echo $SUBSCRIBE_QUERY

echo "--------------------------------------------------------"
echo "subscribe to events"

# cat keeps the stdin open so websocat doesnt close
(echo "$SUBSCRIBE_TRANSFER"; echo "$SUBSCRIBE_QUERY"; cat) | websocat $WSURL | while read msg; do 
    if [[ "$msg" == '{"jsonrpc":"2.0","id":1,"result":{}}' ]] || [[ "$msg" == '{"jsonrpc":"2.0","id":2,"result":{}}' ]]; then
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

    if echo "$msg" | jq '.result.events[].type' | grep -q '"wasm-transfer"'; then
        echo "... received wasm-transfer event! "
        echo $msg 

        echo "... fetching requests"

        REQUESTS=$($CMD query wasm contract-state raw $CONTRACT $(printf '%s' "requests" | hexdump -ve '/1 "%02X"') -o json | jq -r .data | base64 -d)
        STATE=$($CMD query wasm contract-state raw $CONTRACT $(printf '%s' "state" | hexdump -ve '/1 "%02X"') -o json | jq -r .data | base64 -d)

        export ENCLAVE_REQUEST=$(jq -nc --argjson requests "$REQUESTS" --argjson state $STATE '$ARGS.named')
        echo $ENCLAVE_REQUEST | jq .

        export REQUEST_MSG=$(jq -nc --arg message "$ENCLAVE_REQUEST" '$ARGS.named')

        cd $ROOT/cycles-quartz/apps/transfers/enclave

        echo "... executing transfer"
        export UPDATE=$(grpcurl -plaintext -import-path ./proto/ -proto transfers.proto -d "$REQUEST_MSG" '127.0.0.1:11090' transfers.Settlement/Run | jq .message | jq -R 'fromjson | fromjson' | jq -c )

        echo "... submitting update"

        $CMD tx wasm execute $CONTRACT "{\"update\": "$UPDATE" }" --chain-id testing --from admin --node http://$NODE_URL -y

        echo " ... done"
        echo "---------------------------------------------------------"
        echo "... waiting for event"
    elif echo "$msg" | jq '.result.events[].type' | grep -q '"wasm-query_balance"'; then
        echo "... received wasm-query_balance event! "
        echo $msg 

        echo "... fetching state"

        STATE=$($CMD query wasm contract-state raw $CONTRACT $(printf '%s' "state" | hexdump -ve '/1 "%02X"') -o json | jq -r .data | base64 -d)

        # Extract the address from the event
        ADDRESS=$(echo "$msg" | jq -r '.result.events[] | select(.type == "wasm-query_balance") | .attributes[] | select(.key == "address") | .value')

        # Create the enclave request with state and address
        export ENCLAVE_REQUEST=$(jq -nc --argjson state "$STATE" --arg address "$ADDRESS" '$ARGS.named')
        echo $ENCLAVE_REQUEST | jq .

        export REQUEST_MSG=$(jq -nc --arg message "$ENCLAVE_REQUEST" '$ARGS.named')

        cd $ROOT/cycles-quartz/apps/transfers/enclave

        echo "... executing query balance"
        export BALANCE=$(grpcurl -plaintext -import-path ./proto/ -proto transfers.proto -d "$REQUEST_MSG" '127.0.0.1:11090' transfers.Settlement/Query | jq .message | jq -R 'fromjson | fromjson' | jq -c )

        echo "... submitting update"

        $CMD tx wasm execute $CONTRACT "{\"store_balance\": "$BALANCE" }" --chain-id testing --from admin --node http://$NODE_URL -y

        echo " ... done"
        echo "---------------------------------------------------------"
        echo "... waiting for event"
    fi
done
