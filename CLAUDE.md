# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Portmapper is a Rust CLI tool that maps MAC addresses to switch ports via SNMP v2c bulk-walk queries. It queries a network switch using BRIDGE-MIB OIDs and displays port-to-MAC mappings in table, ASCII map, or interactive TUI format.

## Build & Test Commands

```bash
cargo build                    # build (without TUI)
cargo build --features tui     # build with ratatui TUI support
cargo test                     # run all tests (all offline-capable)
cargo test mac_to_string_test  # run a single unit test

# Run against a switch
cargo run -- -c <community> <ip_address>
cargo run -- -c <community> -d ascii <ip_address>              # ASCII map mode
cargo run -- -c <community> -d ascii --resolve-ip <ip_address> # with IP resolution
cargo run --features tui -- -c <community> -d tui <ip_address> # interactive TUI
```

## Architecture

Single-binary CLI with six modules:

- **main.rs** — Entry point, error types (`AppError`/`AppResult` using `thiserror`), OID constants, display mode dispatch, and table printer.
- **cli.rs** — Argument parsing via `clap` v4 derive. Required: `-c <community>`, positional `<IPADDRESS>`. Optional: `-s <sort_by>` (mac|port), `-d <display>` (table|ascii|tui), `--resolve-ip`.
- **mapper.rs** — Core SNMP logic using `snmp2` crate. `bulkwalk()` implements GETBULK walking. `get_port_macs()` parses MAC-to-port mappings. `get_ip_to_mac()` queries ipNetToPhysicalPhysAddress for IP resolution. `bulkwalk_raw()` variant returns raw bytes for binary SNMP values.
- **topology.rs** — Data model (`Device`, `PortEntry`, `SwitchTopology`) and `build()` function that groups MACs by port and joins with optional IP data.
- **display_ascii.rs** — Renders `SwitchTopology` as a box-drawing ASCII diagram with tree connectors for multi-device ports.
- **display_tui.rs** — Feature-gated (`tui` feature) interactive ratatui TUI with port list and device detail panes.

## Key Details

- Dependencies: `snmp2` 0.5, `clap` 4, `thiserror` 2, `eui48` 1.1. Optional: `ratatui` 0.29, `crossterm` 0.28 (behind `tui` feature).
- MAC addresses are encoded as dotted-decimal suffixes of the SNMP OID (e.g., `184.39.235.100.148.192`) and converted to canonical format via `eui48`.
- The `tui` feature is optional to keep the default binary lean. Compile with `--features tui` to enable `--display tui`.
- All current tests are offline-capable (no live SNMP device required).
