# Setting up a Single Node Neutron Testnet

This guide provides instructions for setting up a single node Neutron testnet using Docker and local installation.

> Note - For more detailed instructions, refer to the [official Neutron documentation](https://docs.neutron.org/neutron/build-and-run/neutron-docker).

## Setup Steps

Clone the Neutron repository in your `$HOME` or your preferred repository:
```
git clone -b v4.0.1 https://github.com/neutron-org/neutron.git
cd neutron
```

Build the Docker image:
```
make build-docker-image
```

Start the Docker container:
```
make start-docker-container
```

Monitor the logs:
```
docker ps  # Get the container ID
docker logs -f <neutron-node-container-id>
```

Verify the Docker keyring setup:
```
docker exec -it neutron neutrond query bank balances neutron1qnk2n4nlkpw9xfqntladh74w6ujtulwn6dwq8z --chain-id test-1
```
This should return:
```
balances:
- amount: "100000000000000"
denom: untrn
pagination:
total: "1"
```

Install neutrond locally:
```
make install
```

To setup the local keyring:
```
cd docker/neutrond
make create-local-accounts
```

Verify local keyring setup:
```
neutrond query bank balances neutron1qnk2n4nlkpw9xfqntladh74w6ujtulwn6dwq8z --chain-id test-1
```

This should return:
```
balances:
- amount: "100000000000000"
denom: untrn
pagination:
total: "1"
```

To stop and reset the chain, go back into the neutron source folder from github and run:
```
make stop-docker-container
```

## How accounts are setup on neutron
We use the standard 7 accounts that come from the neutron base docker setup. We have imported those to `docker/neutrond/accounts`. These accounts already exist in the container running the node, and we get them locally by running `make import-local-accounts`. We don't have functions to create or delete accounts, since we want to strictly follow their base docker setup, and thus keep the accounts the exact same. The 7 accounts are:
- `demowallet1`, `demowallet2` and `demowallet3` - These are the accounts you should use for testing. They are seeded with the test token `untrn`, and 2 IBC tokens, `uibcatom` and `uibcusdc`.
- `val1` and `val2` - accounts used to setup the validators for the test network. Seeded only with the test token `untrn`. Use if you need extra accounts beyond the demo wallets.
- `rly1` and `rly2` - accounts used to setup IBC relayers for the test network. Only seeded with `untrn`. Use if you need extra accounts beyond the demo wallets.