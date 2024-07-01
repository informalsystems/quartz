

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

SUBSCRIBE="{\"jsonrpc\":\"2.0\",\"method\":\"subscribe\",\"params\":[\"execute._contract_address = '$CONTRACT' AND wasm-transfer.action = 'user'\"],\"id\":1}"

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


    echo "... received event! "
    echo $msg 

    echo "... fetching requests"

    REQUESTS=$($CMD query wasm contract-state raw $CONTRACT $(printf '%s' "requests" | hexdump -ve '/1 "%02X"') -o json | jq -r .data | base64 -d)
    STATE=$($CMD query wasm contract-state raw $CONTRACT $(printf '%s' "state" | hexdump -ve '/1 "%02X"') -o json | jq -r .data | base64 -d)

    export ENCLAVE_REQUEST=$(jq -nc --argjson requests "$REQUESTS" --argjson state $STATE '$ARGS.named')
    echo $ENCLAVE_REQUEST | jq .

    export REQUEST_MSG=$(jq -nc --arg message "$ENCLAVE_REQUEST" '$ARGS.named')

    cd $ROOT/cycles-quartz/apps/mtcs/enclave

    echo "... executing transfer"
    export UPDATE=$(grpcurl -plaintext -import-path ./proto/ -proto mtcs.proto -d "$REQUEST_MSG" '127.0.0.1:11090' mtcs.Clearing/Run | jq .message | jq -R 'fromjson | fromjson' | jq -c )


    echo "... submitting update"

    $CMD tx wasm execute $CONTRACT "{\"update\": "$UPDATE" }" --chain-id testing --from admin --node http://$NODE_URL -y


    echo " ... done"
    echo "---------------------------------------------------------"
    echo "... waiting for event"
done


