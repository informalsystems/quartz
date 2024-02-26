FROM gramineproject/gramine:v1.5

RUN apt-get update && apt-get install -y jq build-essential protobuf-compiler libprotobuf-dev git

RUN mkdir /root/.ssh/ \
 && touch /root/.ssh/known_hosts \
 && ssh-keyscan github.com >> /root/.ssh/known_hosts

WORKDIR /workdir

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup toolchain install 1.72.0
#ENV RUST_BACKTRACE=1

RUN gramine-sgx-gen-private-key

# This should be associated with an acive IAS SPID in order for
# gramine tools like gramine-sgx-ias-request and gramine-sgx-ias-verify
ARG RA_TYPE=epid
ENV RA_TYPE=$RA_TYPE
ARG RA_CLIENT_SPID=51CAF5A48B450D624AEFE3286D314894
ENV RA_CLIENT_SPID=$RA_CLIENT_SPID
ARG RA_CLIENT_LINKABLE=1
ENV RA_CLIENT_LINKABLE=$RA_CLIENT_LINKABLE

ARG DEBUG=0
ENV DEBUG=$DEBUG
ARG SGX=1
ENV SGX=$SGX

# Copy the quartz directory and build
COPY . ./quartz
WORKDIR /workdir/quartz
RUN mkdir -p .cargo/
RUN echo "net.git-fetch-with-cli = true" > .cargo/config.toml
RUN --mount=type=secret,id=ssh_id,target=/root/.ssh/id_ed25519 cargo build --release

WORKDIR /workdir
COPY quartz.manifest.template ./

# Make and sign the gramine manifest
RUN gramine-manifest  \
    -Dlog_level="error"  \
    -Dhome=${HOME}  \
    -Darch_libdir="/lib/$(gcc -dumpmachine)"  \
    -Dra_type="$RA_TYPE" \
    -Dra_client_spid="$RA_CLIENT_SPID" \
    -Dra_client_linkable="$RA_CLIENT_LINKABLE" \
    -Dquartz_dir="$(pwd)/quartz"  \
    quartz.manifest.template quartz.manifest

RUN gramine-sgx-sign --manifest quartz.manifest --output quartz.manifest.sgx

CMD [ "gramine-sgx-sigstruct-view quartz.sig" ]
