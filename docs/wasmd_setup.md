## CosmWasm Node Setup

Here we describe how to initialize accounts and balances on a CosmWasm
blockchain.

We'll assume you're using `wasmd` but it could be `neutrond` or any other.

We'll also assume your chain ID is `testing`.

Run `wasmd init <your name> --chain-id testing` to initialize the local wasmd
folder.

Now open the file `~/.wasmd/config/client.toml` and change the field
`keyring-backend` from `os` to `test`:

```toml
keyring-backend = "test"
```

Now, finally, we can create a local admin key for your wasmd. You'll use this to
deploy contracts:

```bash
wasmd keys add admin
```

This should output a wasm address.

Now either you will setup a local testnet or use an existing testnet. If its an
existing testnet, you need to fund this account. Send this address to someone
who has access to the admin account for your testnet. If you have access
yourself you can send funds yourself:

```bash
wasmd tx bank send <sender key name> <recipient address> <amount ucosm> --chain-id testing -y
```

If you're setting up your own local testnet, continue with the following:

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

In `~/.wasmd/config/config.toml`, you'll need to make sure the listen address
binds to the public IP (0.0.0.0) and the CORS allows all origins:

```toml
[rpc]
laddr = "tcp://0.0.0.0:26657"
cors_allowed_origins = ["*"]
```

And in `~/.wasmd/config/app.toml`:

```toml
[api]
enable = true
address = "tcp://0.0.0.0:1317"
enabled-unsafe-cors = true
```

Now, finally:

```bash
wasmd start
```

And you should have a chain making blocks!

You can also reduce the block time by lowering `timeout_commit` in
`~/.wasmd/config/config.toml`.

Now that you have the chain running, you can start running the enclave and proxy
in other windows.
