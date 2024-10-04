FROM rust:1.81 AS build

ENV TARGET=wasm32-unknown-unknown
ARG CARGO_FLAGS=""

COPY . /opt

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    protobuf-compiler \
    openssl \
    clang \
    ca-certificates

# Install Go
RUN apt-get update && \
    apt-get install -y --no-install-recommends wget && \
    wget https://go.dev/dl/go1.22.0.linux-amd64.tar.gz && \
    tar -C /usr/local -xzf go1.22.0.linux-amd64.tar.gz && \
    rm go1.22.0.linux-amd64.tar.gz && \
    wget https://github.com/neutron-org/neutron/releases/download/v4.2.4/neutrond-linux-amd64 && \
    chmod +x neutrond-linux-amd64 && \
    cp neutrond-linux-amd64 /usr/local/bin/neutrond && \
    apt-get remove -y wget && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

ENV PATH="/usr/local/go/bin:${PATH}"

RUN go install github.com/fullstorydev/grpcurl/cmd/grpcurl@latest && \
    cargo install websocat

RUN rustup target add "$TARGET"
WORKDIR /opt/crates/cli
RUN cargo build --locked --release ${CARGO_FLAGS}
RUN cp /opt/target/release/quartz /usr/local/bin/

# Create a directory for the app
RUN mkdir -p /opt/examples/transfers
# Create a simple wait-for-it script
RUN echo '#!/bin/bash\n\
while ! nc -z neutron-node 26657; do\n\
  echo "Waiting for neutron-node..."\n\
  sleep 1\n\
done\n\
echo "Neutron node is up - executing command"\n\
exec $@' > /usr/local/bin/wait-for-neutron.sh && chmod +x /usr/local/bin/wait-for-neutron.sh

ENTRYPOINT ["/usr/local/bin/wait-for-neutron.sh", "quartz", "--mock-sgx", "--app-dir", "/opt/examples/transfers", "dev", "--unsafe-trust-latest", "--contract-manifest", "/opt/examples/transfers/contracts/Cargo.toml", "--init-msg", "{\"denom\":\"untrn\"}", "--node-url", "http://neutron-node:26657", "--chain-id", "test-1"]
# "--sk-file", "823ad58093b111e3c41369ca67103cba89e136995bdbd8a86c6b7eef9ca38b00" (val1)
# FROM rust:1.81 AS build

# ENV TARGET=wasm32-unknown-unknown
# ARG CARGO_FLAGS=""

# COPY . /opt

# RUN apt-get update && \
#     apt-get install -y --no-install-recommends \
#     protobuf-compiler \
#     openssl \
#     clang \
#     ca-certificates

# # Install Go
# RUN apt-get update && \
#     apt-get install -y --no-install-recommends wget && \
#     wget https://go.dev/dl/go1.22.0.linux-amd64.tar.gz && \
#     tar -C /usr/local -xzf go1.22.0.linux-amd64.tar.gz && \
#     rm go1.22.0.linux-amd64.tar.gz && \
#     # wget https://github.com/CosmWasm/wasmd/releases/download/v0.44.0/wasmd-v0.44.0-linux-amd64.tar.gz && \
#     wget https://github.com/neutron-org/neutron/releases/download/v4.2.4/neutrond-linux-amd64 && \
#     chmod +x neutrond-linux-amd64 && \
#     cp neutrond-linux-amd64 /usr/local/bin/ && \
#     # tar -C /usr/local/bin -xzf wasmd-v0.44.0-linux-amd64.tar.gz && \
#     # rm wasmd-v0.44.0-linux-amd64.tar.gz && \
#     apt-get remove -y wget && \
#     apt-get clean && \
#     rm -rf /var/lib/apt/lists/*
    

# ENV PATH="/usr/local/go/bin:${PATH}"

# RUN go install github.com/fullstorydev/grpcurl/cmd/grpcurl@latest && \
#     cargo install websocat

# RUN rustup target add "$TARGET"
# WORKDIR /opt/crates/cli
# RUN cargo build --locked --release ${CARGO_FLAGS}
# RUN cp /opt/target/release/quartz /usr/local/bin/

# CMD ["quartz", "--mock-sgx", "--app-dir", "/opt/examples/transfers", "dev", "--unsafe-trust-latest", "--contract-manifest", "/opt/examples/transfers/contracts/Cargo.toml", "--init-msg", "{\"denom\":\"ucosm\"}", "--sk-file", "",]
