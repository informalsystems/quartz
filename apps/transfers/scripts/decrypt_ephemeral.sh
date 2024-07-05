#!/bin/bash

# Check if an encrypted balance is provided
if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <encrypted_balance_hex>"
    exit 1
fi

ENCRYPTED_BALANCE=$1

# Check if eciespy is installed
if ! pip list | grep -q eciespy; then
    echo "eciespy is not installed. Installing now..."
    pip install eciespy
fi

# Extract the private key from wasmd
EPHEMERAL_PRIVKEY=$(wasmd keys export ephemeral_user --unsafe --unarmored-hex)

if [ $? -ne 0 ]; then
    echo "Failed to export private key. Make sure 'ephemeral_user' exists and you've entered the correct password."
    exit 1
fi

# Create a temporary Python script for decryption
TEMP_PYTHON_SCRIPT=$(mktemp)
cat << EOF > "$TEMP_PYTHON_SCRIPT"
from ecies import decrypt
import binascii
import sys

private_key = bytes.fromhex(sys.argv[1])
encrypted = binascii.unhexlify(sys.argv[2])

try:
    decrypted = decrypt(private_key, encrypted)
    print(decrypted.decode())
except Exception as e:
    print(f"Decryption failed: {str(e)}")
EOF

# Run the Python script to decrypt
DECRYPTED=$(python3 "$TEMP_PYTHON_SCRIPT" "$EPHEMERAL_PRIVKEY" "$ENCRYPTED_BALANCE")

echo "Decrypted result:"
echo "$DECRYPTED"

# Clean up
rm "$TEMP_PYTHON_SCRIPT"