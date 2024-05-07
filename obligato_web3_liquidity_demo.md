# Obligato web3 liquidity demo

This demo shows end-to-end integration with Obligato for web3 liquidity (i.e. a native ERC20 token).

## Create obligations and tenders on Obligato

Make sure tenders have backing funds.

## Start blockchain

```
# cd bisenzone-cw-mvp

./scripts/keygen.sh
./scripts/init-node.sh
./scripts/run-node.sh
```

## Build contract

```
./scripts/build-contract.sh
```

## Listen to events (for debugging)

```
websocat ws://127.0.0.1:26657/websocket
{ "jsonrpc": "2.0", "method": "subscribe", "params": ["tm.event='Tx'"], "id": 1 }
```

## Init enclave

### Setup and build

```
# cd tee-mtcs/enclaves/quartz

gramine-sgx-gen-private-key

CARGO_TARGET_DIR=./target cargo build --release
```

### Update enclave trusted hash

Running tm-prover with wrong trusted-hash should print the correct one. Update `quartz-manifest.template` with the
correct hash.

```
# cd tee-mtcs/utils/tm-prover

rm light-client-proof.json
cargo run -- --chain-id testing \
--primary "http://127.0.0.1:26657" \
--witnesses "http://127.0.0.1:26657" \
--trusted-height 1 \
--trusted-hash "5237772462A41C0296ED688A0327B8A60DF310F08997AD760EB74A70D0176C27" \
--contract-address "wasm14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s0phg4d" \
--storage-key "quartz_session" \
--trace-file light-client-proof.json
```

### Start enclave

```
# cd tee-mtcs/enclaves/quartz

gramine-manifest  \
-Dlog_level="error"  \
-Dhome=${HOME}  \
-Darch_libdir="/lib/$(gcc -dumpmachine)"  \
-Dra_type="epid" \
-Dra_client_spid="51CAF5A48B450D624AEFE3286D314894" \
-Dra_client_linkable=1 \
-Dquartz_dir="$(pwd)"  \
quartz.manifest.template quartz.manifest

gramine-sgx-sign --manifest quartz.manifest --output quartz.manifest.sgx
gramine-sgx ./quartz
```

## Send initiate request

```
# cd tee-mtcs/utils/quartz-relayer

./scripts/relay.sh Instantiate
```

## Deploy contract

```
# cd bisenzone-cw-mvp

export INSTANTIATE_MSG='{"msg":{"config":{"mr_enclave":"bf161255b339a8ca6d5b169afe7f30b822714a7d6c6eba6d61c1cc9def76f85f","epoch_duration":{"secs":43200,"nanos":0},"light_client_opts":{"chain_id":"testing","trusted_height":1,"trusted_hash":"2ef0e6f9bddf5deaa6fcd6492c3db26d7c62bffc01b538a958d04376e0b67185","trust_threshold":[2,3],"trusting_period":1209600,"max_clock_drift":5,"max_block_lag":5}}},"attestation":{"report":{"report":{"id":"142707374378750501130725576120664455076","timestamp":"2024-02-29T15:46:05.263386","version":4,"epidPseudonym":"+CUyIi74LPqS6M0NF7YrSxLqPdX3MKs6D6LIPqRG/ZEB4WmxZVvxAJwdwg/0m9cYnUUQguLnJotthX645lAogfJgO8Xg5/91lSegwyUKvHmKgtjOHX/YTbVe/wmgWiBdaL+KmarY0Je459Px/FqGLWLsAF7egPAJRd1Xn88Znrs=","advisoryURL":"https://security-center.intel.com","advisoryIDs":["INTEL-SA-00161","INTEL-SA-00219","INTEL-SA-00289","INTEL-SA-00334","INTEL-SA-00615"],"isvEnclaveQuoteStatus":"CONFIGURATION_AND_SW_HARDENING_NEEDED","platformInfoBlob":"150200650000080000141402040180070000000000000000000D00000C000000020000000000000CB07CE30BB4A946AA0669B60D6D7CF7813846108AB5DD858A80765E46C8BB605BF2FF51CCDBA245C71FE13A3FBA311DA42848622A74666C0AFBE0097EFA44B821A6","isvEnclaveQuoteBody":"AgABALAMAAAPAA8AAAAAAFHK9aSLRQ1iSu/jKG0xSJQAAAAAAAAAAAAAAAAAAAAAFBQCBwGAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABQAAAAAAAAAHAAAAAAAAAL8WElWzOajKbVsWmv5/MLgicUp9bG66bWHBzJ3vdvhfAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACHVXC/KNpA3B+CezmwQ/s4vGzMsdvTa6nx4gDzJVHyvwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACjkLJZtCNPj2FyKOH1Ti91v7L/UJcwp2zdCEasfjkN7gAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"},"reportsig":"ToLIdljjuO8Y0RLon27hCTaPxEmzAaCaSX6FEv5lVDVuG+XzeU+89pllA8sq6UHYcnyJ2FflokL8mJ/YCD0VYcCfYEOuJDGoshc7u55CDTTYWsTyv2kTjNp/y8KZ345hwrYUFQYka2E/wimWytMxk6ZDBLL8jopSofI0qqj5X4LnabrW0gSc75P0A+WuvSlrJpsfCakVcNKf63UP6bHSFJPhqlFJzXdrDisB1dl6/tJjM+3c/3BsXe2itA2r1zPmPcQFB1TnMGfsmXpSicrFoJUMe4nHBzn/6ZQipNqpg1a1XbSu5r+lBhxgxNhC1GWNiDd1/tFcX/X1G9AGPkdzzA=="}}}'

./scripts/deploy-contract.sh artifacts/cw_tee_mtcs.wasm

export CONTRACT="wasm14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s0phg4d"
```

## Create session

```
# cd tee-mtcs/utils/quartz-relayer

./scripts/relay.sh SessionCreate
```

```
# cd bisenzone-cw-mvp

export EXECUTE='{"quartz":{"session_create":{"msg":{"nonce":"ff2f7459bb98c4e7c4e5053cfa121f23c17585a2d1f42ae2abfe20b8303ad17c"},"attestation":{"report":{"report":{"id":"70757578285051774053325998077400915962","timestamp":"2024-02-29T15:47:50.695771","version":4,"epidPseudonym":"+CUyIi74LPqS6M0NF7YrSxLqPdX3MKs6D6LIPqRG/ZEB4WmxZVvxAJwdwg/0m9cYnUUQguLnJotthX645lAogfJgO8Xg5/91lSegwyUKvHmKgtjOHX/YTbVe/wmgWiBdaL+KmarY0Je459Px/FqGLWLsAF7egPAJRd1Xn88Znrs=","advisoryURL":"https://security-center.intel.com","advisoryIDs":["INTEL-SA-00161","INTEL-SA-00219","INTEL-SA-00289","INTEL-SA-00334","INTEL-SA-00615"],"isvEnclaveQuoteStatus":"CONFIGURATION_AND_SW_HARDENING_NEEDED","platformInfoBlob":"150200650000080000141402040180070000000000000000000D00000C000000020000000000000CB018F2D0E3C2D4A18B7D561902131BA8E16F613F9B427369B820423B337EEDF4F11BDEFA694F8708B8D5972474F74D21A6FF397BA4E9B1DA17FCBF8BC3952EFC65","isvEnclaveQuoteBody":"AgABALAMAAAPAA8AAAAAAFHK9aSLRQ1iSu/jKG0xSJQAAAAAAAAAAAAAAAAAAAAAFBQCBwGAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABQAAAAAAAAAHAAAAAAAAAL8WElWzOajKbVsWmv5/MLgicUp9bG66bWHBzJ3vdvhfAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACHVXC/KNpA3B+CezmwQ/s4vGzMsdvTa6nx4gDzJVHyvwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD/L3RZu5jE58TlBTz6Eh8jwXWFotH0KuKr/iC4MDrRfAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"},"reportsig":"BlKi4PkSAciSNkLBLFI4nWG+ifF/+4Qp4t6pg0L44jrra59GVXsPY/ktdMugA99EWGSPrE15znaVKBPMx0/gHzrcPTB7CPiQKxLbFIz3knj+4FO0NtvKTQGsl+/fcuyJKrupoE+iUGNDKHNTSwbLbvp3ZryaLfupdbAncfdn6Qdp8uYvx6i+hZZpAHT3o5HM+47n45kEaG/xZk6sQ7gY71CsEDX3YDI/fvcBHc8HV7JsFhnwfLhlYg4L6KKPkLLDOsMfhwqiFyrMKggeBC+PcOduQL2/+H2w6ojZVoXCbTosABMSs/Xn9MXORZtzEOk0hGmsVyoxuCDVa0o7ML6SMA=="}}}}}'

wasmd tx wasm execute "$CONTRACT" "$EXECUTE" --from alice --chain-id testing -y
```

## Set session pk

```
# cd tee-mtcs/utils/tm-prover

rm light-client-proof.json
cargo run -- --chain-id testing \
--primary "http://127.0.0.1:26657" \
--witnesses "http://127.0.0.1:26657" \
--trusted-height 1 \
--trusted-hash "F5A081271E4BBBDB95DF9DD51316A61D997BDBC4E487BF754C8571E016FDE7E4" \
--contract-address "wasm14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s0phg4d" \
--storage-key "quartz_session" \
--trace-file light-client-proof.json
```

```
# cd tee-mtcs/utils/quartz-relayer

export POP='{"light_client_proof":[{"signed_header":{"header":{"version":{"block":"11","app":"0"},"chain_id":"testing","height":"1","time":"2024-02-29T10:31:41.978364008Z","last_block_id":null,"last_commit_hash":"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855","data_hash":"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855","validators_hash":"93543064D77D740239A8454F2586E92064CF9EBB3701E037B656A29CC3F82819","next_validators_hash":"93543064D77D740239A8454F2586E92064CF9EBB3701E037B656A29CC3F82819","consensus_hash":"048091BC7DDC283F77BFBF91D73C44DA58C3DF8A9CBC867405D8B7F3DAADA22F","app_hash":"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855","last_results_hash":"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855","evidence_hash":"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855","proposer_address":"09A2F2EE18D2D1A8E90B5DCB94072F5F4A2C4D5C"},"commit":{"height":"1","round":0,"block_id":{"hash":"2EF0E6F9BDDF5DEAA6FCD6492C3DB26D7C62BFFC01B538A958D04376E0B67185","parts":{"total":1,"hash":"F4F0C31D6EFD2B8F85FD208A66AC22B1B4DC77732E5F342BA8E56C021E41FBE5"}},"signatures":[{"block_id_flag":2,"validator_address":"09A2F2EE18D2D1A8E90B5DCB94072F5F4A2C4D5C","timestamp":"2024-02-29T10:32:04.83875824Z","signature":"ZCWcsrLqZT4vcGythMYHFWhFJJiQQpcNZtw6MyL/SoiyqfQJipyL01IbCj0vR6MnDJMTEpHesue5B8M7V22MDw=="}]}},"validator_set":{"validators":[{"address":"09A2F2EE18D2D1A8E90B5DCB94072F5F4A2C4D5C","pub_key":{"type":"tendermint/PubKeyEd25519","value":"LHmUOWNnOYDfKI5fEQojrwVE9oMbuwgUgmtnIvV0MmI="},"power":"250","name":null}],"proposer":{"address":"09A2F2EE18D2D1A8E90B5DCB94072F5F4A2C4D5C","pub_key":{"type":"tendermint/PubKeyEd25519","value":"LHmUOWNnOYDfKI5fEQojrwVE9oMbuwgUgmtnIvV0MmI="},"power":"250","name":null},"total_voting_power":"250"},"next_validator_set":{"validators":[{"address":"09A2F2EE18D2D1A8E90B5DCB94072F5F4A2C4D5C","pub_key":{"type":"tendermint/PubKeyEd25519","value":"LHmUOWNnOYDfKI5fEQojrwVE9oMbuwgUgmtnIvV0MmI="},"power":"250","name":null}],"proposer":null,"total_voting_power":"250"},"provider":"fcd91a360d566b908ba6f1f904f689d10d9547b0"},{"signed_header":{"header":{"version":{"block":"11","app":"0"},"chain_id":"testing","height":"3713","time":"2024-02-29T15:49:27.384886052Z","last_block_id":{"hash":"0C87694E172935DE5BE64F9FD5B1DBC1BB31D7FAFC7A809EE4366116F83B4C33","parts":{"total":1,"hash":"93A00009D51AB3EB2DD01DB298779D7D7BDC4B657C1D624A2F32C3648F8C1489"}},"last_commit_hash":"DA4326FA203343C97B143F73460F9B9C19B290D673A0EF00D185497490E96B81","data_hash":"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855","validators_hash":"93543064D77D740239A8454F2586E92064CF9EBB3701E037B656A29CC3F82819","next_validators_hash":"93543064D77D740239A8454F2586E92064CF9EBB3701E037B656A29CC3F82819","consensus_hash":"048091BC7DDC283F77BFBF91D73C44DA58C3DF8A9CBC867405D8B7F3DAADA22F","app_hash":"3314F876642C185D25B97FE2694E08610901F83A310172F7138C967A34087ABB","last_results_hash":"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855","evidence_hash":"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855","proposer_address":"09A2F2EE18D2D1A8E90B5DCB94072F5F4A2C4D5C"},"commit":{"height":"3713","round":0,"block_id":{"hash":"050314F47773A197793915067F76519073D6638E09CB9582183CE4840FB330E4","parts":{"total":1,"hash":"2A09C54244DAB756A28E6F48416010624FE4F2266BB132B1341B8A0DA5DD7A39"}},"signatures":[{"block_id_flag":2,"validator_address":"09A2F2EE18D2D1A8E90B5DCB94072F5F4A2C4D5C","timestamp":"2024-02-29T15:49:32.450052946Z","signature":"ATNZkvCLs7Xd76rjOZ+yoKDgKrBQ16sb8YPFAR/aTc16SecnJ6P5+l6xUCIFN+oETBvfAVa4YI2xI3S8YLjPDQ=="}]}},"validator_set":{"validators":[{"address":"09A2F2EE18D2D1A8E90B5DCB94072F5F4A2C4D5C","pub_key":{"type":"tendermint/PubKeyEd25519","value":"LHmUOWNnOYDfKI5fEQojrwVE9oMbuwgUgmtnIvV0MmI="},"power":"250","name":null}],"proposer":{"address":"09A2F2EE18D2D1A8E90B5DCB94072F5F4A2C4D5C","pub_key":{"type":"tendermint/PubKeyEd25519","value":"LHmUOWNnOYDfKI5fEQojrwVE9oMbuwgUgmtnIvV0MmI="},"power":"250","name":null},"total_voting_power":"250"},"next_validator_set":{"validators":[{"address":"09A2F2EE18D2D1A8E90B5DCB94072F5F4A2C4D5C","pub_key":{"type":"tendermint/PubKeyEd25519","value":"LHmUOWNnOYDfKI5fEQojrwVE9oMbuwgUgmtnIvV0MmI="},"power":"250","name":null}],"proposer":null,"total_voting_power":"250"},"provider":"fcd91a360d566b908ba6f1f904f689d10d9547b0"}],"merkle_proof":{"key":"03f04a313a7349b120c55c99788f12f712176bb3e5926d012d0ea72fa2bbb8505171756172747a5f73657373696f6e","value":"7b226e6f6e6365223a2266663266373435396262393863346537633465353035336366613132316632336331373538356132643166343261653261626665323062383330336164313763222c227075625f6b6579223a6e756c6c7d","proof":{"ops":[{"field_type":"ics23:iavl","key":"A/BKMTpzSbEgxVyZeI8S9xIXa7Plkm0BLQ6nL6K7uFBRcXVhcnR6X3Nlc3Npb24=","data":"CqgDCi8D8EoxOnNJsSDFXJl4jxL3Ehdrs+WSbQEtDqcvoru4UFFxdWFydHpfc2Vzc2lvbhJbeyJub25jZSI6ImZmMmY3NDU5YmI5OGM0ZTdjNGU1MDUzY2ZhMTIxZjIzYzE3NTg1YTJkMWY0MmFlMmFiZmUyMGI4MzAzYWQxN2MiLCJwdWJfa2V5IjpudWxsfRoMCAEYASABKgQAAuQ5IiwIARIFAgTkOSAaISCa6ZlnZD6KFCdS3G0LSqKeQmZzUDOqv+/6obcRCtm1MSIqCAESJgQI5Dkg7j389mvurYpapaBwp4VvU/7O0gF1y9tagD7336BqtKQgIioIARImBhDkOSALv1AwWaxTdZCR/gSLM2SFxpAhG5a2jY7CAXANsuLf/iAiKggBEiYIGOQ5IBQjmmQkjj+0hVfpkibyXuf55xQ8MJy1tkwbocasdU0rICIqCAESJgow5Dkg89sa9DXqDi15yEjAyWGoE4eiPCV9DLy3n0XhSO+IvEsgIiwIARIFDErkOSAaISCaZH/S8tYTLwQ1yn8+ad8iHQ1kfVWLIotWNYC+Olp7uQ=="},{"field_type":"ics23:simple","key":"d2FzbQ==","data":"CqgBCgR3YXNtEiDeXMkl0ecXFKw8UjADjriHIePLr/rC1vCIfI9iyE72sBoJCAEYASABKgEAIiUIARIhAWLU8PgnJ/EMp4BYvtTN9MX/rS70dNQ3ZAzrJLssrLjRIiUIARIhAd3cbonoIZaLIds9Htw+kXa5zoLYQ5vlIHWmlOTgrUraIiUIARIhASJ3XuooVI+LIEPrmir7K/XRBaXJRKOZJkfODoLZz1Ve"}]}}}'
export POP_MSG=$(jq -nc --arg message "$POP" '$ARGS.named')

./scripts/relay.sh SessionSetPubKey "$POP_MSG"
```

```
# cd bisenzone-cw-mvp

export EXECUTE='{"quartz":{"session_set_pub_key":{"msg":{"nonce":"ff2f7459bb98c4e7c4e5053cfa121f23c17585a2d1f42ae2abfe20b8303ad17c","pub_key":"03c43cb8a0d9571737992f91506e029dd78e1fb013f7958945b30d42795cc151a6"},"attestation":{"report":{"report":{"id":"198880017151960330796191614468800194929","timestamp":"2024-02-29T15:51:00.523316","version":4,"epidPseudonym":"+CUyIi74LPqS6M0NF7YrSxLqPdX3MKs6D6LIPqRG/ZEB4WmxZVvxAJwdwg/0m9cYnUUQguLnJotthX645lAogfJgO8Xg5/91lSegwyUKvHmKgtjOHX/YTbVe/wmgWiBdaL+KmarY0Je459Px/FqGLWLsAF7egPAJRd1Xn88Znrs=","advisoryURL":"https://security-center.intel.com","advisoryIDs":["INTEL-SA-00161","INTEL-SA-00219","INTEL-SA-00289","INTEL-SA-00334","INTEL-SA-00615"],"isvEnclaveQuoteStatus":"CONFIGURATION_AND_SW_HARDENING_NEEDED","platformInfoBlob":"150200650000080000141402040180070000000000000000000D00000C000000020000000000000CB01A644DC52273E62A859D438810E4B6237D94319FE8C96097253B09FC50DF84FF102617BE649E2C455304196E99209F0FF1219A3A5EB283240DB87586A7B7E32D","isvEnclaveQuoteBody":"AgABALAMAAAPAA8AAAAAAFHK9aSLRQ1iSu/jKG0xSJQAAAAAAAAAAAAAAAAAAAAAFBQCBwGAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABQAAAAAAAAAHAAAAAAAAAL8WElWzOajKbVsWmv5/MLgicUp9bG66bWHBzJ3vdvhfAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACHVXC/KNpA3B+CezmwQ/s4vGzMsdvTa6nx4gDzJVHyvwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAC/DrzAs8I/ytHHgzwChlZGNbGS8ZXW+t6plngDeV7jJAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"},"reportsig":"n2l59baMENECxUJuisajEM/6sF49KIDHzNvfwY9/zHnk4qcmr96Zf9HfLsv/qyxCKuMQqJBlJ7RMTt0tOM9SWnQhGJjqZlVIGhl2PyYvNW5ThuvFlghaQFOkq26byxbwCEnBbWPG1vkcasUzTYdU2MktTO6pi0z9TtwzT3OEThDa43aRyU2hXoD1kCv71ynwndmv/jM/9QM1Ol6FNLIUfd0sarMoUyqIhTqqVO2KI2AP5EyGeCQqENVVl6G1HsZLDX0l8tez+qcJfIn8QmA5hooA8XQ9wou8n+2CIH3vGiHCGpkhZkbx94yALvZogvjimfgvK7vK2BeQ5pa5fknNkw=="}}}}}'

wasmd tx wasm execute "$CONTRACT" "$EXECUTE" --from alice --chain-id testing -y
```

## Check for session success

```
wasmd query wasm contract-state raw "$CONTRACT" $(printf '%s' "quartz_session" | hexdump -ve '/1 "%02X"') -o json | jq -r .data | base64 -d
# {"nonce":"d3283ed5d646298c27f5ef1726c42bf4853ed7f3d30c905fd3607ecc56903db4","pub_key":"02e4d8bc80d032ad610e4643c3da4235076b4d24335cc4c77592562bdcd62ce1d0"}
```

## Sync obligations

Update the `--epoch-pk` with the on-chain `pub_key` (i.e. the output of the above command).

```
# cd tee-mtcs/utils/cycles-sync

cargo run -- --keys-file keys.json \
            --obligation-user-map-file o_map.json \
            --user "alice" \
            --contract "wasm14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s0phg4d" \
            sync-obligations \
            --epoch-pk "022797b2fded24ee46b864b84d3b4e4b1aca328f3a955693cd6d351419fab52c2b"

wasmd query wasm contract-state raw "$CONTRACT" $(printf '%s' "1/obligations" | hexdump -ve '/1 "%02X"') -o json | jq -r .data | base64 -d
```

## Init clearing

Create a clearing cycle on Obligato (required to be able to upload setoffs to Obligato) and initiate clearing on the
blockchain.

```
# cd bisenzone-cw-mvp

wasmd tx wasm execute "wasm14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s0phg4d" '"init_clearing"' --from alice --chain-id testing -y
```

## Run clearing on enclave

```
# cd tee-mtcs/enclaves/quartz

export OBLIGATIONS='{"4019ac8e19c5a897bf385803acdce5ddfa53847259fc91c442e5b3596d327994":"04f64d59807d33199945f4444bdbab02596030388eecc53cc6e52687f690c635a22b741479fbd5ef383eaa698363c8d59865d29000f3817a6816f7fe5d275101e4aa5f4ad7f7178738b21c7a9d9561bdf8c2bb2639ba9d1e0725bd94213fdd4bea7bf51f077fa7702c197a97433a2ec54db9fd896c512953297b7cc61b31f13bfb0d1161c36e3249971ebb5d3ab50263617351191dca7df2a5d62fa7f10c72d04107a3a25741120cb3c3d217b5c2a375b6eea0f7905fac9d5377f32c34ac75e1a9f176aec46e5a59928de844fac2c10e6f4f91f28db664c7d758f2e1e9ed0a3f842c1289fb0a4677485f096fa6562ce612f2ed1842083b4f6c664473acef23d61bc1628072dc309c9238fcbf3b9aadef399c0e26f3bf03e60b0563ad9367eebce78abf5ca2523743569129999b827cc9ecd50843107646e76b497be26422467b35c6ae8e7ca09650ef9992b52b0d57f2b97ea572cae68e4e3dbb1743eff815a9c9bd13bf1fbf5d62f06ff66dd450d40aca35ae4ae953c277f4b010015ed429a402229d666e78ac85fc21ecc4265c857cf02a8cde3754","95fb2e138574e2764ae83aa66353ec60fd4429a2224044d5913b5ed35ca4ccc7":"0442e2dfabb4dcbe1ee569679235cbba27be57ce28a68e175180ed99e9bd64d549404a365f251402860e4e366c143d85a585efdf3e90e30d0d5c2c763c847bb30a055cd408d7cd8ca678baefc762f1a4d873efe4075f06cbe287bae432de8777d14cbcf4e4889538e5e59a67123aa0e862446e07be72fb516986bac71b9ea43ea52350bef617ff7d6b8bb47a58681a7ec03a11c833da473b7957f9c0ea8dfcc911decf0128c275cf7e9ca3eb9c861511e989342346678aa19f993716d02a79fd8cf24104737f480691c6ece1e789d0db3a55a1d600392b2054ac889ea5cce022f0bff3697a13ed62c90bd8be27994e4bf108651554cf32c4cfb4874938245de145ba367676da28d08ed8e9e89a2e5ca674ac0f24252b18bd3c4030379df506c24e8555a6a44de8ad550905975d2ac2a33935a7519c29e538d457d958d5171e66991b03d9523df2da85ee0239d537829c149f855e091b4e60e2e9df29852690463a03f48de40f5cfec6b20f4973e792e901d49ef23a84102ee17dfa6ad10ca01c16f12d7cfede5304dc3937bc891ce71e6cb7ab0af4a74a","e8cac8651a74b78f8540b9dab34c80d2affc74e8fd4835dc4c81e5d825a4376a":"04bdc1ed858f495e5609d36cb94b695c4c65b2876c6557e2595015e1c42a84e24bd7912736f5224331bf8a5b39390c7ede156d15e54d2fd543af19f5c9dbd59a9fa13fbaecdd862f871bb8aaaa8895360dabf60e6e77e8305c431c6dca7e62bb8f5544b916fb11f0129d87297df120847a648a304892d96fe1116d0fbf04e5db32cb6767e35bc0bdd6933c96754c61d15487068f67b7d0031ec7c9fb158dc6414e2a9d626f2219d3e8cf4dcd465609c8af5fa927a4857c2ad4cbe45c676bc5c121ec25983c8b52150ac8c1d34e6c0c5f3c7df4aea25cdad558cd9d4329b634ef716af250965ab38899fde8e3d795d7d6e3e6aad5b65d925103cbb49cf36663d84a0220a0bff1d1e8be35eaf2b5bb3c93c564defa0684ff838b565ec92b7c2fd0a5cace23ade29e547923699682b2965f8b6f0cdb36ab8a5cc9a4df03df2b13cffc5271a116c7210512c98f9bc37fd67c7a859960b80cc16eacfce4e022cf6bfadf63712d419a7665d258afea73ea5790ceea95b5d7c1e0b865de9afd621899aa71844ce7d03ac0ec1c5271bf704c374e41b6fa132ee9"}'
export REQUEST_MSG=$(jq -nc --arg message "$OBLIGATIONS" '$ARGS.named')

grpcurl -plaintext -import-path ../../enclaves/quartz/proto/ -proto mtcs.proto -d "$REQUEST_MSG" '127.0.0.1:11090' mtcs.Clearing/Run | jq -c '.message | fromjson'
```

## Submit setoffs

```
# cd bisenzone-cw-mvp

export SETOFFS='{"setoffs_enc":{"1afd7dace621f25d0664f60bdb1b20d190557f597098d7bfbf6b51cf6a27170f":"049cb44f86905b802b20878160a4e94411e271973558960876feff80671726b4f50354c1cf54d973e97bcd05e6a031219a83b418a004c779d1af230be750cc3e619c821807e67388e2161b72ee1420281179a7f68baa2bfb7ed52c0f05412ed974a1a358af1241900b9213aeeefd1bceef8b92f0da5524c0bb88049476d8de3da268a182e762c7483e274092ecb1855f14432245c4517115337377a7343be23527e402cf1dec8ae35cb01b23ad45958cee92f111d6f17019b1eaa22f2ee50fcee409912d900e98d8e493698ac2541df202e640b2faec592db0d6d0efb3d53260f98e75876924610c3626275c998028ebefe776686d23930e85980f504f36d8558f55ee584450096339d25c7211f055ec825ed0487e25b2b3919258e90fc3559881623dc9d02439","2c9d9dfdd6a5ad51c6e85c1ed789513a585eff90341de37d6c5a9ef8b7fc1b0a":"042593179b8dd505f6a874f3a4e80fd6ce5c82a14139bf8a69d1f81e786a3841267a1eeda76e61e20d714049b6fb25e7ffb5b2d8e800c2d79783d9297ccb1000a545e7ece8ca197dfbb14add0a1e3519ac592ae1809612dd681dbe81d3c02f788f2cce4f0977d590988f446350e907e1a4c2c86ba8a0efa2221671c71fa4c877d476ed3a680bf69ed6e346c86592e75a9fb9c7d11a3e65531e78fe6f03fcf86b19e1ef40fd3586943f4eb6f3cb37f22eaf2ef2dbad2002497b1fd4794c875729ae9e0df6c3d6dbf8f0123edfbbefa40bcf736e37103fd772b40c5ef4b005f388872eb38f00d331fbbca6e4c385b66688c17cf9895c1d819de408bbc2eafb981f9fbdb8334e99d730b6db5e5c9fe6c8ce8c0f78c5d7cb3953318d588968367a8de8a7ffd0081c4058","37223948bc4dcbad9881f22c97139c22bcdcc3788a31d6f59a13a6e738016780":"04536291584f434ad6549ecd7cfa2ec074da383fd04071de2e3c5a2188c20c78261f550518ea2a7edfb7516bbc85a58f229c76549d7ea2da654c350849fd0df4b297de49847b2757d541f310d118cc7b86f3d9ff6785ce47945d0cb75a9eb79d4e1d6a1460a8dd61b1c65920a62fa599125dc0c5efe7f59e19ff417ba01982ef774d67c8e3cade760bd6bedc9b9a719ca1d397d3be5f8bc9dda0d366c4c9fed89db42a3262a4aa33053ee5b49078dea6513c2465fa0d4875988735efe01193749ddf2549ac73c1901178fee995e5a5227a32476153d4accc2bcd2ce064d84dd7b9fdff2c6c3461731d34ab278a0e3a9c604b77eafede161b64061f9f984b627afd48138a1ec26c2d7d8e6e753218b608215b43f540490297b033777135a9e88c278be7009666712a14"}}'
export EXECUTE=$(jq -nc --argjson submit_setoffs "$SETOFFS" '$ARGS.named')
wasmd tx wasm execute "$CONTRACT" "$EXECUTE" --from alice --chain-id testing -y --gas 2000000

wasmd query wasm contract-state raw "$CONTRACT" $(printf '%s' "1/setoffs" | hexdump -ve '/1 "%02X"') -o json | jq -r .data | base64 -d
```

## Verify CW20 balances

```
wasmd query wasm contract-state smart "$CONTRACT" '{"balance": {"address": "wasm1gjg72awjl7jvtmq4kjqp3al9p6crstpar8wgn5"}}'
wasmd query wasm contract-state smart "$CONTRACT" '{"balance": {"address": "wasm1tawlwmllmnwm950a7uttqlyne3k4774rsnuw6e"}}'
```

## Sync setoffs

```
# cd tee-mtcs/utils/cycles-sync

cargo run -- --keys-file keys.json \
            --obligation-user-map-file o_map.json \
            --user "alice" \
            --contract "wasm14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s0phg4d" \
            sync-set-offs
```
