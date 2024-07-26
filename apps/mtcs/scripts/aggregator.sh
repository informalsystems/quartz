set -eo pipefail

ROOT=${ROOT:-$HOME}
DIR_MTCS="$ROOT/cycles-protocol/quartz-app"
DIR_CYCLES_SYNC="$ROOT/cycles-protocol/packages/cycles-sync/"
DIR_PROTO="$DIR_MTCS/enclave/proto"

OVERDRAFT=wasm199rcvzawgyse89k4smqdn4wp83f3q8rurg9vautppxh5cypydafqk9nt6q

cd $DIR_MTCS

export NODE_URL=143.244.186.205:26657
bash scripts/build.sh

cd $DIR_MTCS/scripts/scripts
CONTRACT=$(RUST_BACKTRACE=1 cargo run --bin deploy | tail -n 1)


echo "------------ built and deployed ------------"

PUB_KEY=$(RUST_BACKTRACE=1 cargo run --bin handshake -- --contract $CONTRACT | tail -n 1)

echo "PUB KEY: '$PUB_KEY'"
echo "------------ shook some hands ------------"

cd $DIR_CYCLES_SYNC
cargo run --bin submit -- --epoch-pk $PUB_KEY --mtcs $CONTRACT --overdraft $OVERDRAFT
echo "cargo run --bin submit -- --epoch-pk $PUB_KEY --mtcs $CONTRACT --overdraft $OVERDRAFT"

echo "------------ submitted obligations ------------"

# add contract to owners list in overdrafts contract
CURRENT_SEQUENCE=$(wasmd query account wasm14qdftsfk6fwn40l0xmruga08xlczl4g05npy70 --node http://$NODE_URL --output json | jq -r .sequence)
WASMD_OUTPUT=$(wasmd tx wasm execute $OVERDRAFT '{"add_owner": {"new": "'$CONTRACT'"}}' --from wasm14qdftsfk6fwn40l0xmruga08xlczl4g05npy70 --node http://$NODE_URL --chain-id testing --yes --sequence $CURRENT_SEQUENCE)

echo $WASMD_OUTPUT
echo "------------ added contract as owner of overdrafts ------------"

cd $DIR_MTCS/scripts/scripts
cargo run --bin listen -- --contract $CONTRACT
