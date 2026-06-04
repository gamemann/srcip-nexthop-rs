#!/bin/bash

CONF_FILE=./local.json
CARGO_FLAGS=(-r)
FLAGS=()
IFACE=eth0

# Parse command line arguments.
while [[ $# -gt 0 ]]; do
    key="$1"
    case $key in
        -d|--debug)
            CARGO_FLAGS=()
            ;;
        -c)
            CONF_FILE="$2"
            shift
            ;;
        --cfg=*)
            CONF_FILE="${key#*=}"
            ;;
        -l|--list)
            FLAGS+=("-l")
            ;;
        -i)
            IFACE="$2"
            shift
            ;;
        --iface=*)
            IFACE="${key#*=}"
            ;;
        --help)
            echo "run.sh"
            echo -e "\t-d --debug - Run in debug mode."
            echo -e "\t-c --cfg=<FILE> - The configuration file to use. Defaults to './local.json'."
            echo -e "\t-l --list - List the configured hops and exit."
            echo -e "\t-i --iface=<IFACE> - The network interface to attach to. Defaults to 'eth0'."

            exit 0

            ;;
    esac
    shift
done

FLAGS+=("-c" "$CONF_FILE" "-i" "$IFACE")

sudo -E ~/.cargo/bin/cargo run $CARGO_FLAGS -- "${FLAGS[@]}"