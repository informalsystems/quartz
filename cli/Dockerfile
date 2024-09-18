FROM rust:1.76 AS build

ENV TARGET=x86_64-unknown-linux-musl

ARG CARGO_FLAGS=""

COPY . /opt

RUN apt-get update && apt-get install -y protobuf-compiler && \
    mkdir -m 0700 /root/.ssh && \
    ssh-keyscan github.com >> /root/.ssh/known_hosts

RUN rustup target add "$TARGET"
WORKDIR /opt/cli
RUN --mount=type=ssh cargo build --locked --release ${CARGO_FLAGS}

#------------------------------------------------------------------------------

FROM debian:bookworm-slim

RUN apt-get update && apt install -y openssl cargo

# Copy the enclave binary we built in the previous stage
COPY --from=build /opt/target/release/quartz /usr/local/bin/

# Copy the transfer app files
COPY --from=build /opt /cycles-quartz

CMD ["quartz"]
