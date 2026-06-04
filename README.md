A simple Rust program that attaches an eBPF program to the [TC](https://man7.org/linux/man-pages/man8/tc.8.html) egress hook of a specified network interface using [Aya](https://aya-rs.dev/book/).

This program maps source IPs to next hops (destination MAC addresses) and redirects packets to the appropriate next hop based on the source IP.

This can be used in specific network setups or when debugging.

## Building & Installing
Install Rust and execute the following commands in the project directory:

```bash
# Install required packages through apt.
sudo apt install -y git curl cmake pkg-config libssl-dev llvm-19-dev libclang-19-dev libpolly-19-dev libelf-dev libpcap-dev

# Install Rust using rustup.
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# We need the Rust stable and nightly toolchains.
rustup install stable
rustup toolchain install nightly --component rust-src

# Install linker.
cargo install bpf-linker
```

Next, clone the repository and change into the project directory:

```bash
git clone https://github.com/gamemann/srcip-nexthop-rs.git

cd srcip-nexthop-rs
```

Now you can build and run the program:

```bash
# Only build project.
cargo build

# Build project for release.
cargo build --release

# Build and run project in dev mode.
cargo run # Will fail if eth0 doesn't exist.

# Run project in release mode.,
cargo run --release

# Run project in release mode on interface 'enp1s0'.
cargo run --release -- -i enp1s0

# If you want to install to $PATH (/usr/bin), please use the included script!
# This also copies the default configuration file to /etc/srcip-nexthop/cfg.json, which you can edit to your needs.
# The systemd service file is also installed to /etc/systemd/system/srcip-nexthop.service, which you can enable and start to run the program as a service.
./install.sh

# Now use it like a normal command!
sudo srcip-nexthop
```

You may also use the [`run.sh`](./run.sh) script to run the program with some default arguments.

## Arguments
Here are the current arguments that the tool supports.

| Argument | Short | Description | Default Value |
| --- | --- | --- | --- |
| `--iface` | `-i` | Overrides the network interface specified in the configuration file. | - |
| `--cfg` | `-c` | The path to the configuration file. | `/etc/srcip-nexthop/cfg.json` |
| `--duration` | `-d` | The duration for which the program should run in seconds before exiting. If not specified, the program will run indefinitely until interrupted. | - |
| `--list` | `-l` | List all configured hops and exit. | `false` |
| `--no-stats` | `-n` | Do not print stats to the console. | `false` |

## Configuration File
The configuration file is a JSON file that specifies the mapping of source IPs to next hops (destination MAC addresses).

| Field | Description | Default Value |
| --- | --- | --- |
| `iface` | The network interface to which the eBPF program will be attached. If not specified, the program will use the default interface specified in the code (e.g., `eth0`). | `eth0` |
| `counters_store` | Optional path to a file where the program will write the counters for each hop. If not specified, counters will not be written to a file. | `/tmp/counters.txt` |
| `hops` | A list of hops, where each hop specifies a source IP and a destination MAC address. | `[]` |

Here is an example configuration file:

```json
{
    "counters_store": "/tmp/counters.txt",
    "hops": [
        {
            "src_ip": "192.168.1.30",
            "dst_mac": "aa:bb:cc:dd:ee:ff"
        },
    ]
}
```

## Credits
* [Christian Deacon](https://github.com/gamemann)