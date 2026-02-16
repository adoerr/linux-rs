FROM ubuntu:24.04

RUN apt update

RUN apt install -y iproute2 libcap2-bin netcat jq termshark

COPY ../../target/release/vpn /vpn
COPY ../../target/release/vpn-conf /vpn-conf

COPY run_docker.sh /run_docker.sh

COPY *.conf /

CMD bash run_docker.sh