## Quartz enclave

```bash
docker build . --tag quartz
docker run -it \
      --device /dev/sgx_enclave \
      --device /dev/sgx_provision \
       -v /var/run/aesmd/aesm.socket:/var/run/aesmd/aesm.socket \
       -v ./data:/workdir/data \
       quartz bash
is-sgx-available
gramine-sgx ./quartz
```
