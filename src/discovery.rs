use std::collections::HashSet;
use crate::mapper;
use crate::topology::{self, NetworkNode};
use crate::{resolve_ips, AppResult};

const DOT1D_TP_FDB_PORT: &str = "1.3.6.1.2.1.17.4.3.1.2";

/// Recursively discover network topology starting from a seed switch.
pub fn discover(seed_ip: &str, community: &str, max_depth: u32) -> AppResult<NetworkNode> {
    let mut visited = HashSet::new();
    discover_recursive(seed_ip, community, max_depth, 0, &mut visited)
}

fn discover_recursive(
    switch_ip: &str,
    community: &str,
    max_depth: u32,
    depth: u32,
    visited: &mut HashSet<String>,
) -> AppResult<NetworkNode> {
    let target = format!("{}:161", switch_ip);
    visited.insert(switch_ip.to_string());

    eprintln!(
        "{}Discovering {} ...",
        "  ".repeat(depth as usize),
        switch_ip
    );

    // Get MAC-to-port mappings
    let mac_ports = mapper::get_port_macs(DOT1D_TP_FDB_PORT, &target, community)
        .unwrap_or_default();

    // Get IP-to-MAC mappings (always needed for recursive discovery)
    let ip_macs = resolve_ips(&target, community);

    // Build this node
    let mut node = topology::build(switch_ip, mac_ports, ip_macs);

    // Get sysDescr for this switch
    node.sys_descr = mapper::snmp_probe(&target, community);

    // If we haven't hit max depth, probe discovered devices for child switches
    if depth < max_depth {
        for port in &mut node.ports {
            for device in &mut port.devices {
                if let Some(ref ip) = device.ip {
                    if visited.contains(ip.as_str()) {
                        continue;
                    }
                    let probe_target = format!("{}:161", ip);
                    if let Some(descr) = mapper::snmp_probe(&probe_target, community) {
                        // This device is an SNMP-enabled switch — recurse
                        match discover_recursive(ip, community, max_depth, depth + 1, visited) {
                            Ok(mut child) => {
                                child.sys_descr = Some(descr);
                                device.child_switch = Some(Box::new(child));
                            }
                            Err(e) => {
                                eprintln!(
                                    "{}  Warning: could not walk {}: {}",
                                    "  ".repeat(depth as usize),
                                    ip,
                                    e
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(node)
}
