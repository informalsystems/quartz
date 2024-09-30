# CosmWasm Binaries: Install and Configure

Quartz expects to interact with a CosmWasm-based blockchain. 
The default/archetypal binary is `wasmd`. We also use `neutrond` for the
testnet.

Here we describe how to get setup from scratch or how to use an existing wasmd
binary/network you have access to.

- [Install](#install)
- [Configure from scratch](#configure-from-scratch)
- [Configure existing](#configure-existing)

## Install

To build from source, first make sure you have Go installed.

Then for `wasmd`:

```bash
git clone https://github.com/cosmwasm/wasmd/
cd wasmd
git checkout v0.45.0
go install ./cmd/wasmd
```

Or for `neutrond`:

```bash
git clone -b main https://github.com/neutron-org/neutron.git
cd neutron
git checkout v4.0.0
make install
```

## Configure From Scratch

We have to initialize a new chain and load it with some accounts.

We'll assume you're using `wasmd` but it could be `neutrond` or any other.

We also have to give the chain a chain ID. We'll use `testing`.

Run 

```bash
wasmd init <your name> --chain-id testing
```

to initialize the local wasmd folder.

Now open the file `~/.wasmd/config/client.toml` and change the field
`keyring-backend` from `os` to `test`:

```toml keyring-backend = "test" ```

Now, finally, we can create a local admin key for your wasmd. You'll use this to
deploy contracts:

```bash 
wasmd keys add admin 
```

This should output a wasm address. 

Now create the genesis file.

```bash 
# generate a second key for the validator 
wasmd keys add validator

# fund both accounts in genesis 
wasmd genesis add-genesis-account admin 100000000000stake,100000000000ucosm 
wasmd genesis add-genesis-account validator 100000000000stake,100000000000ucosm

# sign genesis tx from validator and compose genesis 
wasmd genesis gentx validator 100000000stake --chain-id testing 
wasmd genesis collect-gentxs 
```

Before finally starting the node, for it to work with the front end, you need to
configure CORS.

### Configure CORS

In `~/.wasmd/config/config.toml`, you'll need to make sure the listen address
binds to the public IP (0.0.0.0) and the CORS allows all origins:

```toml [rpc] laddr = "tcp://0.0.0.0:26657" cors_allowed_origins = ["*"] ```

And in `~/.wasmd/config/app.toml`:

```toml [api] enable = true address = "tcp://0.0.0.0:1317" enabled-unsafe-cors =
true ```

Now, finally:

## wasmd start

```bash 
wasmd start ```

And you should have a chain making blocks!

You can also reduce the block time by lowering `timeout_commit` in
`~/.wasmd/config/config.toml`.

Now that you have the chain running, you can start running the enclave and proxy
in other windows.

Return to the [getting started guide](/docs/getting_started.md#installation)

## Configure Existing

If you want to join an existing testnet you either need to setup a node and 
sync that testnet or find a node to use.

You'll also need to setup an account and get it funded.

Assuming you're using the Neutron testnet, create a new account called `admin`:

```bash
neutron keys add admin
```

Now use a faucet or send this address to someone who can give you funds on the tesnet. 

If you have funds yourself you can simply transfer them:

```bash 
neutrond tx bank send <sender key name> <recipient address> <amount>
--chain-id testing -y 
```

One your `admin` account is funded on the network, return to the [getting started guide](/docs/getting_started.md#installation)
