#!/bin/bash

set -eo pipefail

ROOT=${ROOT:-$(git rev-parse --show-toplevel)}

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
    echo -e "${RED}${BOLD}❌ Error: $message${NC}"
    exit 1
}

# Function to print waiting messages
print_waiting() {
    local message=$1
    echo -e "${YELLOW}${BOLD}⏳ $message${NC}"
}


print_header "Instantianting relayer"
print_success "Relayer instantiated successfully."

cd  $ROOT/relayer/
echo $ROOT/apps/transfers/contracts/

INSTANTIATE_MSG=$(./scripts/relayNeutron.sh Instantiate | jq -c '.')

cd $ROOT/apps/transfers/contracts/

bash deploy-contract-Neutrond.sh target/wasm32-unknown-unknown/release/transfers_contract.wasm  "$INSTANTIATE_MSG" | tee output
export CONTRACT=$(cat output | grep Address | awk '{print $NF}' | sed 's/\x1b\[[0-9;]*m//g')