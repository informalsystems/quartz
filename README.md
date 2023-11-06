# The Tendermint light client enclave

This enclave runs tendermint/CometBFT light client verification on a given 'verification trace' using a user specified
root-of-trust. It outputs the new root-of-trust to a sealed file so that it can be used for future instantiations.

## Execution on an SGX machine

This is tested on a local SGX machine, not Azure

```bash
docker build . --tag tmdocker build . --tag tm \
docker run -it --device /dev/sgx_enclave \
      -v /var/run/aesmd/aesm.socket:/var/run/aesmd/aesm.socket \
      -v ./tests:/workdir/tests \
      tm bash
is-sgx-available
gramine-sgx ./tm
```
