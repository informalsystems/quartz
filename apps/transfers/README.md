# Transfers

This is a simple Quartz demo app. It allows users to deposit funds to a contract, transfer them around privately within the contract's encrypted state, 
and withdraw whatever funds they have.

## Run

First set the `NODE_URL` variable to the address of the blockchain node. If it's a local node, set it to `localhost:26657`. If it's a remote node, set it to that node's address.

### Gramine Setup

If you haven't already generated an sgx priv key:

```
gramine-sgx-gen-private-key
```

Build the enclave:

```
cd enclave
CARGO_TARGET_DIR=./target cargo build --release
cd ..
```

The built binary is a grpc server that hosts the transfer application.

Now we need to get the trusted hash to initialize the enclave. Running tm-prover with wrong trusted-hash should print the correct one:

```
cd $HOME/cycles-quartz/utils/tm-prover
rm light-client-proof.json
cargo run -- --chain-id testing \
--primary "http://$NODE_URL" \
--witnesses "http://$NODE_URL" \
--trusted-height 1 \
--trusted-hash "5237772462A41C0296ED688A0327B8A60DF310F08997AD760EB74A70D0176C27" \
--contract-address "wasm14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s0phg4d" \
--storage-key "quartz_session" \
--trace-file light-client-proof.json &> output
cat output | grep found | head -1 | awk '{print $NF}' | sed 's/\x1b\[[0-9;]*m//g' > trusted.hash
export TRUSTED_HASH=$(cat trusted.hash)
```

Note we dump the output of the command into the `output` file, which we then parse to get the trusted hash,
strip of any extra chars, and finally save into the `trusted.hash` file (we'll use this again laster). We also save it to an env var.

Now update the `quartz-manifest.template` with the correct ("found") hash from the previous command:

```
cd $HOME/cycles-quartz/apps/transfers/enclave

sed -i -r "s/(\"--trusted-hash\", \")[A-Z0-9]+(\"])/\1$TRUSTED_HASH\2/" quartz.manifest.template
```

That will overwrite the template file in place, inserting the new hash in place of the old one. 

Now we can start the enclave:

```
gramine-manifest  \
-Dlog_level="error"  \
-Dhome=${HOME}  \
-Darch_libdir="/lib/$(gcc -dumpmachine)"  \
-Dra_type="epid" \
-Dra_client_spid="51CAF5A48B450D624AEFE3286D314894" \
-Dra_client_linkable=1 \
-Dquartz_dir="$(pwd)"  \
-Dtrusted_height="1"  \
-Dtrusted_hash="$TRUSTED_HASH"  \
quartz.manifest.template quartz.manifest

gramine-sgx-sign --manifest quartz.manifest --output quartz.manifest.sgx
gramine-sgx ./quartz
```

### Contract Setup

Now that the enclave is running, we can build and deploy the contract.

Open another window, and set the NODE_URL env variable again.

Now build the contract:

```
cd contracts
bash build.sh
cd .. 
```

Then we can deploy the contract:

```
bash scripts/init.sh
```

Note the deployed contract address and save it into the `CONTRACT` env variable.

Now run the quartz handshake between contract and enclave:

```
bash scripts/handshake.sh $CONTRACT
```

This should output the pubkey and nonce.

### Run the Listener

Finally, we're ready to listen to events from the contract and trigger execution on the enclave:

```
bash scripts/transfer.sh
```

Now we can interact with the contract, and we'll see the events and contract data come through.


### Run the Frontend

Now on your own machine, checkout the https://github.com/informalsystems/cycles-hackathon-app.


Make sure to create a `.env.local` file and set the contract address and TEE pubkey (see the output from `init.sh` and `handshake.sh`). For example:

```
#.env.local
NEXT_PUBLIC_TRANSFERS_CONTRACT_ADDRESS=wasm1ch9ed27cdu3a4fkx37gnagm7jcthj0rggnmmjwwwe4xhwmk0d65q8fn9pz
NEXT_PUBLIC_ENCLAVE_PUBLIC_KEY=030c25e39743fd4c7553d87873919281d567b5c328fb903cbfbe9541518736a2d2
```

Install and run the app:

```
npm install -f
npm run dev
```

Note the frontend app is currently hardcoded to talk to our remote digital ocean node. 

Make sure you have Keplr installed in your browser and you should now be able to use the app!

You may have to go to "Manage Chain Visibility" in Keplr settings to add the `My Testing Chain`.

Then you should be able to deposit, transfer, and withdraw using different Keplr accounts. And everything will get processed automatically by the transfer.sh script we have running on the enclave host!




