ARG WASMD_VERSION=v0.44.0

FROM cosmwasm/wasmd:${WASMD_VERSION}

# Increase the amount of ucosm given to accounts
RUN sed -i  's/1000000000/12000000000000/g' /opt/setup_wasmd.sh

# Set up wasmd
RUN /opt/setup_wasmd.sh

# Configure the validator for single-node operation within a Docker container
RUN <<EOF
sed -i 's/enabled-unsafe-cors = false/enabled-unsafe-cors = true/g' /root/.wasmd/config/app.toml &&
sed -i 's/address = \"tcp:\/\/localhost:1317\"/address = \"tcp:\/\/0.0.0.0:1317\"/g' /root/.wasmd/config/app.toml
sed -i 's/address = \"localhost:909/address = \"0.0.0.0:909/g' /root/.wasmd/config/app.toml
sed -i 's/enable = false/enable = true/g' /root/.wasmd/config/app.toml
sed -i 's/rpc-max-body-bytes = 1000000$/rpc-max-body-bytes = 1000000000/g' /root/.wasmd/config/app.toml
sed -i 's/laddr = \"tcp:\/\/127.0.0.1:26657\"/laddr = \"tcp:\/\/0.0.0.0:26657\"/g' /root/.wasmd/config/config.toml
sed -i 's/cors_allowed_origins = \[\]/cors_allowed_origins = \[\"*\"\]/g' /root/.wasmd/config/config.toml
sed -i 's/max_body_bytes = 1000000$/max_body_bytes = 1000000000/g' /root/.wasmd/config/config.toml
sed -i 's/max_tx_bytes = 1048576$/max_tx_bytes = 104857600/g' /root/.wasmd/config/config.toml
EOF

CMD ["/opt/run_wasmd.sh"]