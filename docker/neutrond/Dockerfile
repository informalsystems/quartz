FROM ubuntu:20.04 as builder

RUN apt-get update && apt-get install -y wget

RUN wget https://github.com/neutron-org/neutron/releases/download/v4.2.4/neutrond-linux-amd64 && \
    chmod +x neutrond-linux-amd64 && \
    mv neutrond-linux-amd64 /usr/local/bin/neutrond

FROM ubuntu:20.04

RUN apt-get update && apt-get install -y bash

COPY --from=builder /usr/local/bin/neutrond /usr/local/bin/neutrond

# Copy the account setup script
COPY setup_accounts.sh /opt/neutron/

# Make the script executable
RUN chmod +x /opt/neutron/setup_accounts.sh

# Set the entrypoint to run the setup script and start neutrond
# ENTRYPOINT ["/bin/bash", "-c", "/opt/neutron/setup_accounts.sh && neutrond start --minimum-gas-prices '0.0053untrn'"]
# Set the entrypoint to run the setup script
ENTRYPOINT ["/bin/bash", "-c", "/opt/neutron/setup_accounts.sh"]
# FROM golang:1.22.6 as builder

# WORKDIR /app
# RUN git clone https://github.com/neutron-org/neutron.git .
# RUN apt-get update && apt-get install -y make
# RUN make build
# RUN make install


# FROM alpine:latest

# RUN apk add --no-cache bash

# COPY --from=builder /go/bin/neutrond /usr/local/bin/neutrond
# COPY --from=builder /app/network /opt/neutron/network

# # Copy the account setup script
# COPY setup_accounts.sh /opt/neutron/

# # Make the script executable
# RUN chmod +x /opt/neutron/setup_accounts.sh

# # Set the entrypoint to run the setup script and start neutrond
# ENTRYPOINT ["/bin/bash", "-c", "/opt/neutron/setup_accounts.sh && neutrond start --minimum-gas-prices '0.0053untrn'"]
# FROM golang:1.22.6 as builder

# WORKDIR /app
# RUN git clone https://github.com/neutron-org/neutron.git .
# RUN apt-get update && apt-get install -y make
# RUN make build
# RUN make install 
# RUN make create-local-accounts


# # Install necessary dependencies
# RUN apk add --no-cache git make bash


# FROM alpine:latest


# COPY --from=builder /app/build/neutrond /usr/local/bin/neutrond

# COPY --from=builder . /opt/neutron
# # COPY --from=build /go/bin/neutrond /usr/local/bin/neutrond


# # CMD ["/bin/bash", "-c", "/setup_accounts.sh && neutrond start --chain-id test-1"]

# CMD bash /opt/neutron/network/init.sh && \
#     bash /opt/neutron/network/init-neutrond.sh && \
#     bash /opt/neutron/network/start.sh#