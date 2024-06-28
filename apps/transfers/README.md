# Transfers

This is a simple Quartz demo app. It allows users to deposit funds to a contract, transfer them around privately within the contract's encrypted state, 
and withdraw whatever funds they have.

## Run

First set the `NODE_URL` variable to the address of the blockchain node. If it's a local node, set it to `localhost:26657`. If it's a remote node, set it to that node's address.

The `scripts` dir contains some bash scripts to help run the app. 
These scripts should be replaced by a new `quartz` tool. See [issue](https://github.com/informalsystems/cycles-quartz/issues/61).

### Build the Binaries

Build the enclave binary and the smart contract binary:

```
bash scripts/build.sh
```

### Configure and Run Gramine

Setup and sign the Gramine config, and then start the gramine process, which will run the 
grpc server that hosts the transfer application.

```
bash scripts/start.sh
```

The enclave binary is now running, waiting for commands.


### Contract Setup

With the enclave running in one window, open another window to deploy the contract and start the listener.

In the new window, set the NODE_URL env variable again (eg. `export NODE_URL=143.244.186.205:26657`)

Now we can deploy the contract:

```
bash scripts/deploy.sh
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
bash scripts/listen.sh $CONTRACT
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




