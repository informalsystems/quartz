## Gramine experiments using MTCS

This is a Dockerfile for replaying the mtcs experiment in gramine, starting from the manifest file from https://github.com/informalsystems/cofi-private/issues/104
The starting point for the Dockerfile is the Gramine-based from Revm Relay hackathon. https://github.com/amiller/gramine-sgx-revm/

The point of this is to emphasize the verification process that can be completed even without SGX, by reproducing the MRENCLAVE and inspecting remote attestation quotes.

## Replicating the MRENCLAVE build (no SGX required)

The following will build mtcs, then freeze all dependencies from the docker environment into the gramine manifest, and finally display the resulting MRENCLAVE
```bash
docker build . --tag mtcs
docker run -it -v ./data:/workdir/data mtcs
```

Let's see how long this remains reproducible:
```
     mr_enclave: fa9149158c693b09e83480b48c2e7344c941aadca6d5829834f2af9f2690435e
```

## Execution on an SGX machine

This is tested on a local SGX machine, not Azure

```bash
docker run -it --device /dev/sgx_enclave \
       -v /var/run/aesmd/aesm.socket:/var/run/aesmd/aesm.socket \
       -v ./data:/workdir/data \
       mtcs bash
is-sgx-available
gramine-sgx ./mtcs
cat mtcs/data/micro-set-offs.out
```