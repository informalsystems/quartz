# Quartz Neutrond image

This folder contains a `Dockerfile` that helps build a single-node [neutrond]
for use in testing your Quartz application.

It facilitates the creation of a Docker image with 4 accounts pre-loaded, each
having a small amount of `untrn` preloaded from genesis for experimentation.

- `admin`
- `alice`
- `bob`
- `charlie`

These accounts' details are stored in clear text in the [data/accounts](./data/accounts/)
folder.

**Note: this image is _NOT_ intended to be used in production.**

## Updating neutrond binary

We are using a special neutrond binary that allows to run the node in dev mode to be able to work with quartz correctly. To build this binary you need to follow next steps:

```bash
# Clone the target neutrond version you want to build:
git clone  --depth 1 --branch v4.2.4 https://github.com/neutron-org/neutron.git /neutron && cd ./neutron

# Build the binary using special build flag:
make build-static-linux-amd64 BUILD_TAGS=skip_ccv_msg_filter

# It will generate the binary in `neutron/build/neutrond-linux-amd64`, copy that file:
mv ./build/neutrond-linux-amd64 {PATH_CYCLES_POCKET}/docker/neutrond/data/neutrond

# And now you can rebuild the docker image with new binary

```

## Importing the account keys

As previously mentioned, the [data/accounts](./data/accounts/) folder contains all of
the necessary material to construct the public/private keypairs of the accounts.

A convenient helper target is provided in [`/Makefile`](./Makefile) to facilitate
importing of these accounts into a local `neutrond` configuration (i.e. on your
host machine, outside of the Docker container). This will allow you to transact
on behalf of any of those accounts from outside of the Docker container.

**NB**: For this to work, you will need the same version of `neutrond` installed on
your local machine as what is built into the `neutrond` Docker image.

```bash
make import-local-accounts
```

To check that the accounts have been imported correctly, on your host machine
run:

```bash
# List all keys available in your local neutrond configuration
neutrond keys list --keyring-backend=test
```
