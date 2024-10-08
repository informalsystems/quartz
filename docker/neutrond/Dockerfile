FROM phusion/baseimage:focal-1.2.0

RUN curl -L -s -O --compressed https://github.com/neutron-org/neutron/releases/download/v4.2.4/neutrond-linux-amd64 && \
    chmod +x neutrond-linux-amd64 && \
    mv neutrond-linux-amd64 /usr/local/bin/neutrond

COPY data /root/

ENTRYPOINT ["/root/entrypoint.sh"]
