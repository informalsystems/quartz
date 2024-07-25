#!/bin/bash

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
ROOT=${HOME}
DIR_QUARTZ="$ROOT/cycles-quartz"
DIR_QUARTZ_APP="$DIR_QUARTZ/apps/transfers"
DIR_QUARTZ_ENCLAVE="$DIR_QUARTZ_APP/enclave"
DIR_QUARTZ_TM_PROVER="$DIR_QUARTZ/utils/tm-prover"

NODE_URL=${NODE_URL:-127.0.0.1:26657}
CMD="neutrond --node http://$NODE_URL"
QUARTZ_PORT="${QUARTZ_PORT:-11090}"

print_header "Quartz Setup and Launch"
print_message $CYAN "QUARTZ_PORT is set to: $QUARTZ_PORT"

print_message $BLUE "Setting trusted hash and height"
CHAIN_STATUS=$($CMD status)
TRUSTED_HASH=$(echo "$CHAIN_STATUS" | jq -r .sync_info.latest_block_hash)
TRUSTED_HEIGHT=$(echo "$CHAIN_STATUS" | jq -r .sync_info.latest_block_height)
print_message $YELLOW "Trusted Hash: $TRUSTED_HASH"
print_message $YELLOW "Trusted Height: $TRUSTED_HEIGHT"

cd "$DIR_QUARTZ_APP"
echo "$TRUSTED_HASH" > trusted.hash
echo "$TRUSTED_HEIGHT" > trusted.height
print_success "Trusted hash and height saved"

if [ -n "$MOCK_SGX" ]; then
    print_header "Running in MOCK_SGX mode"
    cd $DIR_QUARTZ_ENCLAVE
    print_message $BLUE "Launching enclave without Gramine..."
    ./target/release/quartz-app-transfers-enclave --chain-id "test-1" --trusted-height "$TRUSTED_HEIGHT" --trusted-hash "$TRUSTED_HASH"
    exit
fi

print_header "Configuring Gramine"
cd "$DIR_QUARTZ_ENCLAVE"

print_message $BLUE "Generating private key (if it doesn't exist)"
gramine-sgx-gen-private-key > /dev/null 2>&1 || :

print_message $BLUE "Creating manifest"
gramine-manifest  \
-Dlog_level="error"  \
-Dhome="$HOME"  \
-Darch_libdir="/lib/$(gcc -dumpmachine)"  \
-Dra_type="epid" \
-Dra_client_spid="51CAF5A48B450D624AEFE3286D314894" \
-Dra_client_linkable=1 \
-Dquartz_dir="$(pwd)"  \
-Dtrusted_height="$TRUSTED_HEIGHT"  \
-Dtrusted_hash="$TRUSTED_HASH"  \
-Dgramine_port="$QUARTZ_PORT" \
quartz.manifest.template quartz.manifest

print_message $BLUE "Signing manifest"
gramine-sgx-sign --manifest quartz.manifest --output quartz.manifest.sgx

print_header "Starting Gramine"
print_message $GREEN "Launching Quartz with Gramine-SGX..."
gramine-sgx ./quartz