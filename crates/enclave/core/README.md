## Quartz enclave

### Enclave usage

```bash
gramine-sgx-gen-private-key

CARGO_TARGET_DIR=./target cargo build --release

gramine-manifest  \
    -Dlog_level="error"  \
    -Dhome=${HOME}  \
    -Darch_libdir="/lib/$(gcc -dumpmachine)"  \
    -Dra_type="dcap" \
    -Dra_client_spid="51CAF5A48B450D624AEFE3286D314894" \
    -Dra_client_linkable=1 \
    -Dquartz_dir="$(pwd)"  \
    quartz.manifest.template quartz.manifest

gramine-sgx-sign --manifest quartz.manifest --output quartz.manifest.sgx
gramine-sgx ./quartz
```

### CLI usage

```bash
cargo run -- --chain-id testing \
    --trusted-height 1 \
    --trusted-hash "A1D115BA3A5E9FCC12ED68A9D8669159E9085F6F96EC26619F5C7CEB4EE02869"
```
