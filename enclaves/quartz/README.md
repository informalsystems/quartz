## Quartz enclave

### Enclave usage

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

### CLI usage

```bash
cargo run -- --chain-id testing \    
    --mr-enclave "fa9149158c693b09e83480b48c2e7344c941aadca6d5829834f2af9f2690435e" \
    --trusted-height 1 \
    --trusted-hash "A1D115BA3A5E9FCC12ED68A9D8669159E9085F6F96EC26619F5C7CEB4EE02869"
```
