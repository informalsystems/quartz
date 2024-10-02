# CosmWasm SGX TcbInfo Smart Contract

Standalone smart contract for storage and verification of `TcbInfo`s for Intel SGX. The contract ensures that
TcbInfos are kept up-to-date so other contracts can query the latest TcbInfo state using the quote's `fmspc` during
remote attestation verification to ensure the attesting enclave setup is up-to-date.

## Overview

The contract provides the following functionalities:

- Instantiate: Initialize the contract with a root certificate.
- Execute: Store and verify TcbInfo along with the provided certificate and optional timestamp.
- Query: Retrieve the latest TcbInfo using the FMSPC.

## Usage (with wasmd)

- Submit a new `TcbInfo` for a specific `fmspc`

```shell
export EXECUTE='{
  "tcb_info": "{\"tcbInfo\":{ /* ... */ },\"signature\":\"647bac99371750892415557b838237839e52b02afe027a43322fe661f4a1a693b04a82717120d74bccf2b3787bf7e9ecbe44caa06e6e532b7a68a21b2765663d\"}
  "certificate": "-----BEGIN CERTIFICATE-----\\n /* ... */ \\n-----END CERTIFICATE-----"
}'
wasmd tx wasm execute "$CONTRACT" "$EXECUTE" --from alice --chain-id testing -y
```

- Query the latest `TcbInfo` by `fmspc`

```shell
wasmd query wasm contract-state smart "$CONTRACT" '{"get_tcb_info": {"fmspc": "00906ED50000"}}'
```
