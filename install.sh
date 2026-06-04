#!/bin/bash

TYPE=release # release or debug

if [ "$TYPE" = "release" ]; then
    sudo -E ~/.cargo/bin/cargo build --release
else
    sudo -E ~/.cargo/bin/cargo build
fi

# Install to /usr/bin
sudo cp -f target/$TYPE/srcip-nexthop-rs /usr/bin/srcip-nexthop

# Make sure we have a default config.
sudo mkdir -p /etc/srcip-nexthop

if [ ! -f /etc/srcip-nexthop/cfg.json ]; then
    sudo cp -nf ./cfg.ex.json /etc/srcip-nexthop/cfg.json
fi

# Copy the systemd service file.
sudo cp -nf ./srcip-nexthop.service /etc/systemd/system/srcip-nexthop.service