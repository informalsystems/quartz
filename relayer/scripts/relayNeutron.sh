#!/bin/bash

set -eo pipefail


usage() {
    echo "Usage: $0 <REQUEST> <REQUEST_MSG>"
    echo "    <REQUEST>: Instantiate | SessionCreate | SessionSetPubKey"
    exit 1
}


ROOT=${ROOT:-$(git rev-parse --show-toplevel)}
DIR_PROTO="$ROOT/core/quartz-proto/proto"
IAS_API_KEY="669244b3e6364b5888289a11d2a1726d"
RA_CLIENT_SPID="51CAF5A48B450D624AEFE3286D314894"
QUOTE_FILE="/tmp/${USER}_test.quote"
REPORT_FILE="/tmp/${USER}_datareport"
REPORT_SIG_FILE="/tmp/${USER}_datareportsig"

REQUEST="$1"
REQUEST_MSG=${2:-"{}"}

# Use the QUARTZ_PORT environment variable if set, otherwise default to 11090
QUARTZ_PORT="${QUARTZ_PORT:-11090}"


# query the gRPC quartz enclave service
ATTESTED_MSG=$(grpcurl -plaintext -import-path "$DIR_PROTO" -proto quartz.proto -d "$REQUEST_MSG" "127.0.0.1:$QUARTZ_PORT" quartz.Core/"$REQUEST" | jq -c '.message | fromjson')

# parse out the quote and the message
QUOTE=$(echo "$ATTESTED_MSG" | jq -c '.quote')
MSG=$(echo "$ATTESTED_MSG" | jq 'del(.quote)')



if [ -n "$MOCK_SGX" ]; then
    case "$REQUEST" in
        "Instantiate")
            jq -nc --argjson msg "$MSG" --argjson "attestation" "$QUOTE" '$ARGS.named'
            ;;
        "SessionCreate" | "SessionSetPubKey")
            REQUEST_KEY=$(echo "$REQUEST" | perl -pe 's/([A-Z])/_\L$1/g;s/^_//')
            jq -nc --argjson quartz "$(jq -nc --argjson "$REQUEST_KEY" "$(jq -nc \
                --argjson msg "$MSG" --argjson attestation "$QUOTE" '$ARGS.named')" \
                '$ARGS.named')" '$ARGS.named'
            ;;
        *)
            usage
            ;;
    esac
    exit
fi

# clear tmp files from previous runs
rm -f "$QUOTE_FILE" "$REPORT_FILE" "$REPORT_SIG_FILE"

# request the IAS report for EPID attestations
echo -n "$QUOTE" | xxd -r -p - > "$QUOTE_FILE"
gramine-sgx-ias-request report -g "$RA_CLIENT_SPID" -k "$IAS_API_KEY" -q "$QUOTE_FILE" -r "$REPORT_FILE" -s "$REPORT_SIG_FILE" > /dev/null 2>&1
REPORT=$(cat "$REPORT_FILE")
REPORTSIG=$(cat "$REPORT_SIG_FILE" | tr -d '\r')

#echo "$QUOTE"
#echo "$REPORT"
#echo "$REPORTSIG"

case "$REQUEST" in
"Instantiate")
    jq -nc --argjson msg "$MSG" --argjson "attestation" \
        "$(jq -nc --argjson report "$(jq -nc --argjson report "$REPORT" --arg reportsig "$REPORTSIG" '$ARGS.named')" '$ARGS.named')" \
        '$ARGS.named' ;;

    "SessionCreate" | "SessionSetPubKey")
    REQUEST_KEY=$(echo "$REQUEST" | perl -pe 's/([A-Z])/_\L$1/g;s/^_//')
    jq -nc --argjson quartz "$(jq -nc --argjson "$REQUEST_KEY" "$(jq -nc --argjson  msg "$MSG" --argjson attestation \
                "$(jq -nc --argjson report "$(jq -nc --argjson report "$REPORT" --arg reportsig "$REPORTSIG" '$ARGS.named')" '$ARGS.named')" \
                '$ARGS.named')" '$ARGS.named')" '$ARGS.named' ;;

*)
    usage ;;
esac