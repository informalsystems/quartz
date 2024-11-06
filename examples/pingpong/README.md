# Ping Pong

This is a minimal Quartz demo app to serve as a scaffold to learn and build off of.

It allows users to submit a message encrypted to the enclave's pubkey, 
and receive a response from the enclave encrypted to their own pubkey.
<!-- 
See the [getting started guide](/docs/getting_started.md) to get it quickly
setup on a local testnet without SGX or on a real testnet using an Azure SGX
node.

See this [video of an early demo of the app at the Flashbots
TEE-salon](https://www.youtube.com/watch?v=3Tv6k02zvBc&t=2517s).
 -->

# Step by Step

1) Run

```
quartz --mock-sgx dev --unsafe-trust-latest --contract-manifest contracts/Cargo.toml
```

2. Copy the contract address from the output and paste it onto line 46 of the `send_message.rs` script. 
3. Copy the pub key from the handshake and paste it onto line 18 of `send_message.rs`. 

The pubkey in the `Ping` struct will be the user's pubkey, which you can leave untouched. 

4. Next, send this transaction to "Ping" the enclave.
```
cargo run --bin send_message
```

5. Watch the logs from `quartz dev`. You should see the decrypted message printed. 

Now let's find the enclave's response. 

6. Query the contract for its messages
```
neutrond query wasm contract-state smart $CONTRACT '{
  "get_all_messages": {}
}' -o json
```

7. Copy the value from, what should be, the only entry in the map. 

8. Go to `send_message.rs` and paste the hex value onto line 67. Run `decrypt_enclave_response` to see the decrypted output. 