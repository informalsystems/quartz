# Transfers

This is a simple Quartz demo app. It allows users to deposit funds to a contract, transfer them around privately within the contract's encrypted state, 
and withdraw whatever funds they have.

## Setup

### Install Rust

We only have this working so far with rust v1.76.0 since we're running against wasmd v0.44. 

Install rust by executing a script from the internet (😅):

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

You may want to exit and start a new terminal session to get the rust toolchain on your path.

Now downgrade rust to v1.76.0:

```
rustup install 1.76.0
rustup default 1.76.0
```

Check the version with `cargo version`.

Finally add the wasm target:

```
rustup target add wasm32-unknown-unknown
```

And you should be good to go!

### Install Other Tools

You need a few other Go based tools. You should already have go.

First add the `~/go/bin` to your path by adding this line to the end of your `~/.bashrc`:

```
export PATH="${PATH}:${HOME}/go/bin"
```

Then `source ~/.bashrc`. Now we can install some stuff.

You need grpcurl:

```
go install github.com/fullstorydev/grpcurl/cmd/grpcurl@latest
```

You need wasmd v0.44.0:

```
git clone https://github.com/cosmwasm/wasmd/
cd wasmd
git checkout v0.44.0
go install ./cmd/wasmd
```

Check that both work by running `grpcurl` and `wasmd`.

Finally, you neeed `websocat`:

```
cargo install websocat
```

### Fetch the Repo

While the repos are private you will need an ssh key registered on github in order to fetch them. Run `ssh-keygen` and just follow the defaults. On Github, go to Settings -> SSH and GPG Keys -> New SSH Key. Run `cat ~/.ssh/id_rsa.pub` on the machine and copy and paste the result into Github.

This key will now have full access to your github account so its a good idea to remove it when you're done and re-add everytime you need to push/pull the repo. 

Now clone the repo:

```
git clone ssh://git@github.com/informalsystems/cycles-quartz
cd cycles-quartz
```

Tada!

### Setup Wasmd Accounts

Run `wasmd init <your name>` to initialize the local wasmd folder.

Now open the file `~/.wasmd/config/client.toml` and change the field `keyring-backend` from `os` to `test`:

```
keyring-backend = "test"
```

Now, finally, we can create a local admin key for your wasmd. You'll use this to deploy contracts:

```
wasmd keys add admin
```

This should output a wasm address. Now you need to fund this account. 
Send this address to someone who has access to the admin account for your testnet. If you have access yourself you can send funds yourself:

```
wasmd tx bank send <sender key name> <recipient address> <amount ucosm> --chain-id testing -y 
```

## Run

First set the `NODE_URL` variable to the address of the blockchain node. If it's a local node, set it to `localhost:26657`. If it's a remote node, set it to that node's address (eg. `export NODE_URL=143.244.186.205:26657`).

The `scripts` dir contains some bash scripts to help run the app. 
These scripts should be replaced by a new `quartz` tool. See [issue](https://github.com/informalsystems/cycles-quartz/issues/61).

Note: to build/run on a non-SGX machine you must set the `MOCK_SGX` env var for all bash scripts below
```
# 1st Terminal
$ export MOCK_SGX=1
$ bash scripts/build.sh
$ bash scripts/start.sh

# 2nd Terminal
$ export MOCK_SGX=1
$ bash scripts/deploy.sh
$ ...
```

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

Copy the `.env.example` file to `.env.local`:

```
cp .env.example .env.local
```


and set the relevant fields. You should have the contract address and TEE pubkey from the output of the `deploy.sh` and `handshake.sh` scripts, respectfully. The chain id is probably `testing` and the IP address for the URLs is probably `143.244.186.205`.
Modify accordingly. For example:

```
#.env.local
NEXT_PUBLIC_TRANSFERS_CONTRACT_ADDRESS=wasm1ch9ed27cdu3a4fkx37gnagm7jcthj0rggnmmjwwwe4xhwmk0d65q8fn9pz
NEXT_PUBLIC_ENCLAVE_PUBLIC_KEY=030c25e39743fd4c7553d87873919281d567b5c328fb903cbfbe9541518736a2d2
NEXT_PUBLIC_CHAIN_ID=testing
NEXT_PUBLIC_CHAIN_RPC_URL=http://143.244.186.205:26657
NEXT_PUBLIC_CHAIN_REST_URL=http://143.244.186.205:1317
```

Install and run the app:

```
npm install -f
npm run dev
```

You can now open the app in http://localhost:3000/. 

Make sure you have Keplr installed in your browser and you should now be able to use the app!

You may have to go to "Manage Chain Visibility" in Keplr settings to add the `My Testing Chain` so you can see your balance.

You will also need to fund any keplr account by sending funds from the CLI using your `admin` account. 

Then you should be able to deposit, transfer, and withdraw using different Keplr accounts. And everything will get processed automatically by the transfer.sh script we have running on the enclave host!





