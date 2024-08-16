# Quartz Enclave Build/Run Image for SGX

This folder contains the basis for a multi-stage Docker image that:

1. Builds the enclave
2. Takes the binary from the build stage and embeds it in the [Gramine Docker
   image][gramine-docker], such that it can run on an SGX-enabled machine.

## Requirements

- Docker

The build process itself does not require an SGX-capable processor, but running
the image does.

## Setup

**TODO: Remove this subsection once all necessary subcomponents are
open-sourced.**

Before building the image, you will need to ensure that the image has access to
a public/private keypair that will allow access to the private dependencies
needed by the build process.

For example, you could generate a public/private keypair as follows. **NB: This
keypair must _not_ be password-protected, since it needs to be accessible in an
unsupervised manner during the Docker image build.**

```bash
# Should generate ~/.ssh/{id_ed25519,id_ed25519.pub}
ssh-keygen -t ed25519
```

Both the public and private keys must be copied into a `.secrets` folder in the
root of this repository prior to building the image.

```bash
# From the root of the cycles-quartz repo
mkdir -p .secrets
cp ~/.ssh/id_ed25519* .secrets/
```

You will, of course, need to make sure that you have added the public key to
your GitHub account in your SSH key settings before these keys will be useful.
Once you have built the image, you can delete the keys and remove them from your
GitHub account.

## Building

As an example, to build a Docker image for the transfers app's enclave:

```bash
# From the root of the cycles-quartz repository
docker build \
   --build-arg ENCLAVE_DIR=apps/transfers/enclave \
   --build-arg ENCLAVE_BIN=quartz-app-transfers-enclave \
   --build-arg TRUSTED_HEIGHT=1234 \
   --build-arg TRUSTED_HASH=0123456789abcdef \
   -t informaldev/transfers-enclave \
   -f ./docker/enclave-sgx/Dockerfile \
   .
```

This builds an image tagged `informaldev/transfers-enclave:latest`.

The following build arguments are important:

- `ENCLAVE_DIR` - The relative path, from the root of the repository, to the
  enclave source code.
- `ENCLAVE_BIN` - The filename of the enclave binary (usually defined in the
  `Cargo.toml` file).
- `TRUSTED_HEIGHT` - The trusted height of the chain to build into the image for
  the enclave light client.
- `TRUSTED_HASH` - The trusted hash of the chain to build into the image for the
  enclave light client.

## Running

On an SGX-enabled machine:

```bash
# The devices need to be mounted into the container.
docker run --rm -it \
   --device /dev/sgx_enclave \
   --device /dev/sgx_provision \
   informaldev/transfers-enclave
```

[gramine-docker]: https://hub.docker.com/r/gramineproject/gramine
