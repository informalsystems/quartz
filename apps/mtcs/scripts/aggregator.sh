set -eo pipefail

ROOT=${ROOT:-$HOME}
DIR_MTCS="$ROOT/cycles-quartz/apps/mtcs"
DIR_CYCLES_SYNC="$ROOT/cycles-quartz/utils/cycles-sync/"
DIR_PROTO="$DIR_MTCS/enclave/proto"

cd $DIR_MTCS

export NODE_URL=143.244.186.205:26657
bash scripts/build.sh

source scripts/deploy.sh

echo "yeah"
echo $CONTRACT

echo "------------ built and deployed ------------"

cd $DIR_MTCS
handshake_output=$(bash scripts/handshake.sh $CONTRACT)
# Extract the last line of the handshake.sh output
last_line=$(echo "$handshake_output" | tail -n 1)
# Extract the pub_key value from the JSON
PUB_KEY=$(echo "$last_line" | jq -r '.pub_key')

echo "PUB KEY: '$PUB_KEY'"
echo "------------ shook some hands ------------"

cd $DIR_CYCLES_SYNC
cargo run --bin submit $PUB_KEY $CONTRACT "false"

echo "------------ submitted obligations ------------"

#add contract to owners list in overdrafts contract
sleep 1
CURRENT_SEQUENCE=$(wasmd query account wasm14qdftsfk6fwn40l0xmruga08xlczl4g05npy70 --node http://$NODE_URL --output json | jq -r .sequence)
WASMD_OUTPUT=$(wasmd tx wasm execute wasm1huhuswjxfydydxvdadqqsaet2p72wshtmr72yzx09zxncxtndf2sqs24hk '{"add_owner": {"new": "'$CONTRACT'"}}' --from wasm14qdftsfk6fwn40l0xmruga08xlczl4g05npy70 --node http://$NODE_URL --chain-id testing --sequence $CURRENT_SEQUENCE)

echo $WASMD_OUTPUT
echo "------------ added contract as owner of overdrafts ------------"

cd $DIR_MTCS

bash scripts/listen.sh $CONTRACT
