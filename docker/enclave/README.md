# Quartz Enclave Build/Run Image

This folder contains the basis for a multi-stage Docker image that:

1. Builds the enclave
2. Takes the binary from the build stage and embeds it in the [Gramine Docker
   image][gramine-docker], such that it can run on an SGX-enabled machine.

## Requirements

- Docker

The build process itself does not require an SGX-capable processor, but running
the image does.

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
   -f ./docker/enclave/Dockerfile \
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
# The /dev/sgx_enclave device needs to be mounted into the container.
docker run --rm -it \
   --device /dev/sgx_enclave \
   informaldev/transfers-enclave
```

[gramine-docker]: https://hub.docker.com/r/gramineproject/gramine