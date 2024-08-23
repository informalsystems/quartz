#!/bin/bash

set -eo pipefail

usage() {
    echo "Usage: $0 <REQUEST> <REQUEST_MSG>"
    echo "    <REQUEST>: Instantiate | SessionCreate | SessionSetPubKey"
    exit 1
}

ROOT=${ROOT:-$HOME}
DIR_QUARTZ=$(git rev-parse --show-toplevel)
DIR_PROTO="$DIR_QUARTZ/core/quartz-proto/proto"

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
    exit 0
fi

case "$REQUEST" in
    "Instantiate")
        jq -nc --argjson msg "$MSG" --argjson "attestation" \
            "$(jq -nc --argjson collateral "$COLLATERAL" '$ARGS.named')" \
            '$ARGS.named'
        ;;

    "SessionCreate" | "SessionSetPubKey")
        REQUEST_KEY=$(echo "$REQUEST" | perl -pe 's/([A-Z])/_\L$1/g;s/^_//')
        jq -nc --argjson quartz "$(jq -nc --argjson "$REQUEST_KEY" "$(jq -nc --argjson msg "$MSG" --argjson attestation \
                    "$(jq -nc --argjson collateral "$COLLATERAL" '$ARGS.named')" \
                    '$ARGS.named')" '$ARGS.named')" '$ARGS.named'
        ;;

    *)
        usage
        ;;
esac
