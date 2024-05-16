#!/bin/bash

# Generate keys for testing.

set -euo pipefail

wasmd keys add admin
wasmd keys add alice
wasmd keys add bob
wasmd keys add charlie
