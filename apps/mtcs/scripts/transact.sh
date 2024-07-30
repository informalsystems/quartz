
OVERDRAFT=wasm1huhuswjxfydydxvdadqqsaet2p72wshtmr72yzx09zxncxtndf2sqs24hk
CMD='wasmd --node http://$NODE_URL'

# users

ALICE=wasm124tuy67a9dcvfgcr4gjmz60syd8ddaugl33v0n
BOB=wasm1ctkqmg45u85jnf5ur9796h7ze4hj6ep5y7m7l6

# query alice

$CMD query wasm contract-state smart $OVERDRAFT '{"balance": {"user": "'$ALICE'"}}' 

# query bob

$CMD query wasm contract-state smart $OVERDRAFT '{"balance": {"user": "'$BOB'"}}' 

# make obligation from alice to bob for 10 

# $CMD tx wasm execute $CONTRACT '{"submit_obligation_msg": {"ciphertext": "", "digest": ""}}' --from $CONTRACT --chain-id testing

# make bob acceptance to overdraft for 10

# make alice tender from overdraft for 10

# init clearing

$CMD tx wasm execute $CONTRACT '"init_clearing"' --from $CONTRACT --chain-id testing

# wait for 2 sec
sleep 2

# query alice

$CMD query wasm contract-state smart $OVERDRAFT '{"balance": {"user": "'$ALICE'"}}' 

# query bob

$CMD query wasm contract-state smart $OVERDRAFT '{"balance": {"user": "'$BOB'"}}' 

