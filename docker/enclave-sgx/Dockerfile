FROM rust:1.80-alpine AS build

ARG CARGO_FLAGS=""
# By default we assume that there is an "enclave" directory in the root of the
# Quartz app that contains the enclave's code.
ARG ENCLAVE_DIR="enclave"

COPY . /opt/src
WORKDIR /opt/src

# TODO: Remove once the Quartz dependencies are open-sourced
RUN apk update && \
    apk add --no-cache git openssh && \
    mkdir -m 0700 /root/.ssh && \
    cp .secrets/* /root/.ssh/ && \
    chmod 0600 /root/.ssh/* && \
    chmod 0644 /root/.ssh/*.pub && \
    ssh-keyscan github.com >> /root/.ssh/known_hosts

# System dependencies for building our binary
RUN apk update && \
    apk add --no-cache build-base protobuf-dev

RUN cd /opt/src/${ENCLAVE_DIR} && \
    CARGO_TARGET_DIR=./target cargo build --release ${CARGO_FLAGS}

# TODO: Remove once the Quartz dependencies are open-sourced
RUN rm -rf /root/.ssh/

#------------------------------------------------------------------------------

FROM gramineproject/gramine:1.7-jammy

ARG ENCLAVE_DIR="enclave"
# By default we assume that the enclave binary's name is just "enclave".
ARG ENCLAVE_BIN="enclave"
ARG TRUSTED_HEIGHT
ARG TRUSTED_HASH

RUN apt update && \
    apt install -y build-essential

# Copy the enclave binary we built in the previous stage
COPY --from=build /opt/src/${ENCLAVE_DIR}/target/release/${ENCLAVE_BIN} /opt/enclave/bin/enclave
COPY --from=build /opt/src/${ENCLAVE_DIR}/quartz.manifest.template /opt/enclave/

WORKDIR /opt/enclave

# TODO - update entire file to use DCAP, not EPID
RUN gramine-sgx-gen-private-key > /dev/null 2>&1 && \
    gramine-manifest  \
        -Dlog_level="error"  \
        -Dhome="/opt/enclave"  \
        -Denclave_dir="/opt/enclave"  \
        -Denclave_executable="/opt/enclave/bin/enclave" \
        -Darch_libdir="/lib/$(gcc -dumpmachine)"  \
        -Dra_type="epid" \
        -Dra_client_linkable=1 \
        -Dtrusted_height="${TRUSTED_HEIGHT}"  \
        -Dtrusted_hash="${TRUSTED_HASH}"  \
        -Dgramine_port=11090 \
        quartz.manifest.template quartz.manifest && \
    gramine-sgx-sign --manifest quartz.manifest --output quartz.manifest.sgx

CMD ["/restart_aesm.sh && gramine-sgx ./quartz"]