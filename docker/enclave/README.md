# Quartz Enclave Build/Run Image

This folder contains the basis for a multi-stage Docker image that:

1. Builds the enclave
2. Takes the binary from the build stage and embeds it in the [Gramine Docker
   image][gramine-docker], such that it can run on an SGX-enabled machine.

## Requirements

- Docker

The build process itself does not require an SGX-capable processor, but running
the image does.

[gramine-docker]: https://hub.docker.com/r/gramineproject/gramine
