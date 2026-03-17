# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Portmapper is a Rust CLI tool that maps MAC addresses to switch ports via SNMP v2c bulk-walk queries. It queries a network switch using BRIDGE-MIB OIDs and displays a table of port-to-MAC mappings.

## Build & Test Commands

```bash
cargo build           # build
cargo run -- -c <community> <ip_address>   # run against a switch
cargo test            # run all tests (note: SNMP tests require a live device)
cargo test mac_to_string_test   # run a single unit test
```

## Architecture

Single-binary CLI with three modules:

- **main.rs** — Entry point, error types (`AppError`/`AppResult` using the `failure` crate), result printing/sorting logic, and SNMP OID constants (`DOT1D_TP_FDB_PORT`, `IP_NET_TO_PHYSICAL_PHYS_ADDRESS`).
- **cli.rs** — Argument parsing via `clap` v2. Required args: `-c <community>` and positional `<IPADDRESS>`. Optional: `-s <sort_by>` (mac|port).
- **mapper.rs** — Core SNMP logic. `bulkwalk()` implements recursive SNMP GETBULK walking. `get_port_macs()` parses OID-encoded MAC addresses from walk results into `MacPort` structs. Also contains inline `#[test]` functions and helpers (`convert_oid`, `convert_to_mac`, `convert_to_hex`).

## Key Details

- Uses older crate versions: `clap` 2.x, `failure` for error handling, `snmp` 0.2, `eui48` 0.4.
- MAC addresses are encoded as dotted-decimal suffixes of the SNMP OID (e.g., `184.39.235.100.148.192`) and converted to canonical format via `eui48`.
- Tests in `mapper.rs` that perform SNMP walks require a live network device (hardcoded as `TEST_TARGET`/`TEST_COMMUNITY`); `mac_to_string_test` is the only offline-capable test.
- `src/test.rs` exists but is not wired into the module tree (not declared in `main.rs`).
