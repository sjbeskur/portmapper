# Portmapper Quick Start

Portmapper maps MAC addresses to switch ports via SNMP v2c. It can also resolve IPs and recursively discover downstream switches to build a network topology tree.

## Build

```bash
cargo build --release
```

## Basic Usage

Query a single switch (table output):

```bash
portmapper -c <community> <switch_ip>
```

Example:

```bash
portmapper -c public 192.168.88.1
```

## Display Modes

```bash
# Plain table (default)
portmapper -c public 192.168.88.1

# ASCII tree diagram
portmapper -c public -d ascii 192.168.88.1

# Interactive TUI (arrow keys to navigate, q to quit)
portmapper -c public -d tui 192.168.88.1
```

## Resolve IP Addresses

Add `--resolve-ip` to look up IPs for each MAC via the switch's ARP table:

```bash
portmapper -c public --resolve-ip 192.168.88.1
portmapper -c public --resolve-ip -d ascii 192.168.88.1
```

## Recursive Network Discovery

Use `-r` to automatically probe discovered devices for SNMP and walk any switches found, building a full topology tree:

```bash
portmapper -c public -r 192.168.88.1
portmapper -c public -r -d ascii 192.168.88.1
portmapper -c public -r -d tui 192.168.88.1
```

Limit recursion depth (default is 3):

```bash
portmapper -c public -r --max-depth 2 192.168.88.1
```

## Sorting

Sort output by MAC address (default) or port number:

```bash
portmapper -c public -s port 192.168.88.1
portmapper -c public -s mac 192.168.88.1
```

## All Options

```
Usage: portmapper [OPTIONS] --community <COMMUNITY> <IP_ADDRESS>

Arguments:
  <IP_ADDRESS>              Target IPv4 address to query

Options:
  -c, --community <STR>     SNMP v2 community string (required)
  -s, --sort-by <COL>       Sort by column: mac (default) or port
  -d, --display <MODE>      Display mode: table (default), ascii, or tui
      --resolve-ip          Resolve IP addresses from ARP table
  -r, --recursive           Recursively discover SNMP-enabled switches
      --max-depth <N>       Max recursion depth [default: 3]
  -h, --help                Print help
  -V, --version             Print version
```
