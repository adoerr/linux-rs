#!/bin/bash

# This is run_client.sh

sudo setcap cap_net_admin=eip target/release/vpn

target/release/vpn  172.18.0.2:1967 &

pid=$!

sudo ip addr add 10.8.0.3/24 dev vpn0
sudo ip link set up dev vpn0
sudo ip link set dev vpn0 mtu 1400

trap "kill $pid" INT TERM

wait $pid