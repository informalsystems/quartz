#!/bin/bash

# Run a previously initialized wasmd node.

set -uo pipefail

docker volume inspect wasmd_data >/dev/null 2>&1
RESULT=$?
if [ $RESULT -eq 1 ]; then
  echo "wasmd isn't initialized - run 'init-node.sh' first"
  exit 1
fi

echo "Starting wasmd node"

docker run --rm -it -p 26657:26657 -p 26656:26656 -p 1317:1317 -p 9090:9090 \
  --mount type=volume,source=wasmd_data,target=/root \
  --name wasmd \
  cosmwasm/wasmd:v0.44.0  \
  /bin/sh -c "sed -i 's/enabled-unsafe-cors = false/enabled-unsafe-cors = true/g' /root/.wasmd/config/app.toml;
              sed -i 's/address = \"tcp:\/\/localhost:1317\"/address = \"tcp:\/\/0.0.0.0:1317\"/g' /root/.wasmd/config/app.toml;
              sed -i 's/address = \"localhost:909/address = \"0.0.0.0:909/g' /root/.wasmd/config/app.toml;
              sed -i 's/enable = false/enable = true/g' /root/.wasmd/config/app.toml;
              sed -i 's/rpc-max-body-bytes = 1000000$/rpc-max-body-bytes = 1000000000/g' /root/.wasmd/config/app.toml;
              sed -i 's/laddr = \"tcp:\/\/127.0.0.1:26657\"/laddr = \"tcp:\/\/0.0.0.0:26657\"/g' /root/.wasmd/config/config.toml;
              sed -i 's/cors_allowed_origins = \[\]/cors_allowed_origins = \[\"*\"\]/g' /root/.wasmd/config/config.toml;
              sed -i 's/max_body_bytes = 1000000$/max_body_bytes = 1000000000/g' /root/.wasmd/config/config.toml;
              sed -i 's/max_tx_bytes = 1048576$/max_tx_bytes = 104857600/g' /root/.wasmd/config/config.toml;
              /opt/run_wasmd.sh"
