FEATURES=

if [ -n "$MOCK_SGX" ]; then
    echo "MOCK_SGX is set. Adding mock-sgx feature."
    FEATURES="--features=mock-sgx"
fi

RUSTFLAGS='-C link-arg=-s' cargo wasm $FEATURES
