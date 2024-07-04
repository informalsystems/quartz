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

# Attestation constants
IAS_API_KEY="669244b3e6364b5888289a11d2a1726d"
RA_CLIENT_SPID="51CAF5A48B450D624AEFE3286D314894"
QUOTE_FILE="/tmp/${USER}_test.quote"
REPORT_FILE="/tmp/${USER}_datareport"
REPORT_SIG_FILE="/tmp/${USER}_datareportsig"


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

    # TODO - Some reason this is saying ERROR when its fine, will fix
    #if echo "$msg" | sed 's/"log":"\[.*\]"/"log":"<invalid_json>"/' | jq 'has("error")' > /dev/null; then
     #   echo "... error msg $msg"
     #   echo "---------------------------------------------------------"
     #   echo "... waiting for event"
     #   continue
    #fi 

    CLEAN_MSG=$(echo "$msg" | sed 's/"log":"\[.*\]"/"log":"<invalid_json>"/' | jq '.result.events')
    echo "CLEAN" $CLEAN_MSG

    if echo "$CLEAN_MSG" | grep -q 'wasm-transfer'; then
        echo "... received wasm-transfer event!"
        echo "... fetching requests"

        REQUESTS=$($CMD query wasm contract-state raw $CONTRACT $(printf '%s' "requests" | hexdump -ve '/1 "%02X"') -o json | jq -r .data | base64 -d)
        STATE=$($CMD query wasm contract-state raw $CONTRACT $(printf '%s' "state" | hexdump -ve '/1 "%02X"') -o json | jq -r .data | base64 -d)

        export ENCLAVE_REQUEST=$(jq -nc --argjson requests "$REQUESTS" --argjson state $STATE '$ARGS.named')
        echo $ENCLAVE_REQUEST | jq .

        export REQUEST_MSG=$(jq -nc --arg message "$ENCLAVE_REQUEST" '$ARGS.named')

        cd $ROOT/cycles-quartz/apps/transfers/enclave

        # Get the update message from the gRPC response
        UPDATE_MSG=$(grpcurl -plaintext -import-path ./proto/ -proto transfers.proto -d "$REQUEST_MSG" '127.0.0.1:11091' transfers.Settlement/Run | jq -r '.message | fromjson | .msg')

        # Create the UpdateMsg structure
        export UPDATE=$(jq -n \
                        --argjson msg "$UPDATE_MSG" \
                        '{update: $msg}')

        echo "UpdateMsg:"
        echo $UPDATE | jq .

        echo "... submitting update"
        $CMD tx wasm execute $CONTRACT "$(echo $UPDATE | jq -c .)" --chain-id testing --from admin --node http://$NODE_URL -y

        # TODO - add back in once attestations are figured out
        # echo "... executing transfer"
        # export ATTESTED_MSG=$(grpcurl -plaintext -import-path ./proto/ -proto transfers.proto -d "$REQUEST_MSG" '127.0.0.1:11091' transfers.Settlement/Run | jq .message | jq -R 'fromjson | fromjson' | jq -c )
        # # echo $UPDATE #| jq '.msg'
        # # echo $UPDATE | jq '.msg'
        # QUOTE=$(echo "$ATTESTED_MSG" | jq -c '.attestation')
        # MSG=$(echo "$ATTESTED_MSG" | jq -c '.msg')
        # # echo "quote"
        # # echo $QUOTE
        # echo $MSG

        # echo -n "$QUOTE" | xxd -r -p - > "$QUOTE_FILE"
        # gramine-sgx-ias-request report -g "$RA_CLIENT_SPID" -k "$IAS_API_KEY" -q "$QUOTE_FILE" -r "$REPORT_FILE" -s "$REPORT_SIG_FILE" > /dev/null 2>&1
        # REPORT=$(cat "$REPORT_FILE")
        # REPORTSIG=$(cat "$REPORT_SIG_FILE" | tr -d '\r')


        # echo "... submitting update"


        # export EXECUTE=$(jq -nc --argjson update "$(jq -nc --argjson msg "$MSG" --argjson attestation \
        #     "$(jq -nc --argjson report "$(jq -nc --argjson report "$REPORT" --arg reportsig "$REPORTSIG" '$ARGS.named')" '$ARGS.named')" \
        #     '$ARGS.named')" '$ARGS.named')

        # echo $EXECUTE | jq '.'

        
        # $CMD tx wasm execute "$CONTRACT" "$EXECUTE" --from admin --chain-id testing -y --gas 2000000


        # $CMD tx wasm execute $CONTRACT "{\"update\": "$UPDATE" }" --chain-id testing --from admin --node http://$NODE_URL -y

        echo " ... done"
        echo "---------------------------------------------------------"
        echo "... waiting for event"
    elif echo "$CLEAN_MSG" | grep -q 'wasm-query_balance'; then
        echo "... received wasm-query_balance event!"
        echo "... fetching state"

        STATE=$($CMD query wasm contract-state raw $CONTRACT $(printf '%s' "state" | hexdump -ve '/1 "%02X"') -o json | jq -r .data | base64 -d)

        # Extract the address from the event
        ADDRESS=$(echo "$msg" | sed 's/"log":"\[.*\]"/"log":"<invalid_json>"/' | jq -r '.result.events["message.sender"]'[0])

        EPHEMERAL_PUBKEY=$(echo "$msg" | sed 's/"log":"\[.*\]"/"log":"<invalid_json>"/' | jq -r '.result.events["wasm-query_balance.emphemeral_pubkey"]'[0])


        # Create the enclave request with state and address
        export ENCLAVE_REQUEST=$(jq -nc --argjson state "$STATE" --arg address "$ADDRESS" --arg ephemeral_pubkey "$EPHEMERAL_PUBKEY" '$ARGS.named')
        echo $ENCLAVE_REQUEST | jq .

        export REQUEST_MSG=$(jq -nc --arg message "$ENCLAVE_REQUEST" '$ARGS.named')
        echo $REQUEST_MSG | jq .

        cd $ROOT/cycles-quartz/apps/transfers/enclave

        echo "... executing query balance"

        # TODO - Uncomment this out when I get attested messages working
        # # Get the entire msg object, including the attestation
        # ATTESTED_MSG=$(grpcurl -plaintext -import-path ./proto/ -proto transfers.proto -d "$REQUEST_MSG" '127.0.0.1:11091' transfers.Settlement/Query | jq -r '.message | fromjson')

        # # Create the RawQueryResponseMsg structure with address inside the msg
        # export BALANCE=$(jq -n \
        #                 --arg address "$ADDRESS" \
        #                 --argjson msg "$ATTESTED_MSG" \
        #                 '{msg: ($msg.msg + {address: $address}), quote: $msg.quote}')

        # echo "RawQueryResponseMsg:"
        # echo $BALANCE | jq .

        # echo "... submitting update"
        # $CMD tx wasm execute $CONTRACT "{\"query_response\": $(echo $BALANCE | jq -c .) }" --chain-id testing --from admin --node http://$NODE_URL -y

        # Get the encrypted balance from the gRPC response
        ENCRYPTED_BAL=$(grpcurl -plaintext -import-path ./proto/ -proto transfers.proto -d "$REQUEST_MSG" '127.0.0.1:11091' transfers.Settlement/Query | jq -r '.message | fromjson | .msg.encrypted_bal')

        # Create the QueryResponseMsg structure
        export BALANCE=$(jq -n \
                        --arg address "$ADDRESS" \
                        --arg encrypted_bal "$ENCRYPTED_BAL" \
                        '{address: $address, encrypted_bal: $encrypted_bal}')

        echo "QueryResponseMsg:"
        echo $BALANCE | jq .

        echo "... submitting update"
        $CMD tx wasm execute $CONTRACT "{\"query_response\": $(echo $BALANCE | jq -c .)}" --chain-id testing --from admin --node http://$NODE_URL -y

        echo " ... done"
        echo "---------------------------------------------------------"
        echo "... waiting for event"
    fi
done
