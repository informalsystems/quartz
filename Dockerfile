FROM gramineproject/gramine:v1.5

RUN apt-get update && apt-get install -y jq build-essential

WORKDIR /workdir

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup toolchain install 1.72.0

RUN gramine-sgx-gen-private-key

# This should be associated with an acive IAS SPID in order for
# gramine tools like gramine-sgx-ias-request and gramine-sgx-ias-verify
ENV RA_CLIENT_SPID=51CAF5A48B450D624AEFE3286D314894
ENV RA_CLIENT_LINKABLE=1

# Copy the mtcs directory and build
COPY mtcs ./mtcs
WORKDIR /workdir/mtcs
RUN cargo build --release

WORKDIR /workdir
COPY mtcs.manifest.template ./

COPY data/micro-set-offs.csv mtcs/data

# Make and sign the gramine manifest
RUN gramine-manifest -Dlog_level="error" -Dhome=${HOME} -Darch_libdir="/lib/$(gcc -dumpmachine)" -Dmtcs_dir="$(pwd)/mtcs" -Dtestname="micro-set-offs" mtcs.manifest.template mtcs.manifest
RUN gramine-sgx-sign --manifest mtcs.manifest --output mtcs.manifest.sgx

CMD [ "gramine-sgx-sigstruct-view mtcs.sig" ]
