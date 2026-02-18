docker run --name vpn-test \
           --rm --network=vpn-test --cap-add=NET_ADMIN \
           --device=/dev/net/tun vpn-test:latest &