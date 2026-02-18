#!/bin/bash

# This is run_server.sh

setcap 'cap_net_admin=eip'  ./vpn
./vpn &

pid=$!

ip addr add 10.8.0.1/24 dev vpn0
ip link set up dev vpn0
ip link set dev vpn0 mtu 1400

trap "kill $pid" INT TERM

wait $pid