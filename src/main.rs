use thiserror::Error;

const APPVERSION: &str = env!("CARGO_PKG_VERSION");
const APPNAME: &str = env!("CARGO_PKG_NAME");

mod cli;
mod discovery;
mod display_ascii;
#[cfg(feature = "tui")]
mod display_tui;
mod mapper;
mod topology;

// BRIDGE-MIB::dot1dTpFdbPort
const DOT1D_TP_FDB_PORT: &str = "1.3.6.1.2.1.17.4.3.1.2";

// IP-MIB::ipNetToPhysicalPhysAddress (newer, not always supported)
const IP_NET_TO_PHYSICAL_PHYS_ADDRESS: &str = "1.3.6.1.2.1.4.35";

// RFC1213-MIB::ipNetToMediaPhysAddress (classic ARP table, widely supported)
const IP_NET_TO_MEDIA_PHYS_ADDRESS: &str = "1.3.6.1.2.1.4.22.1.2";

fn main() -> AppResult<()> {
    let args = cli::process_args();
    let community = &args.community;

    let node = if args.recursive {
        // Recursive discovery mode
        match discovery::discover(&args.ip_address, community, args.max_depth) {
            Ok(n) => n,
            Err(e) => {
                eprintln!("Error during discovery: {}", e);
                return Ok(());
            }
        }
    } else {
        // Single switch mode
        let ip_addr = format!("{}:161", &args.ip_address);

        let mac_ports = match mapper::get_port_macs(DOT1D_TP_FDB_PORT, &ip_addr, community) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error querying MAC/port mappings: {}", e);
                return Ok(());
            }
        };

        let ip_macs = if args.resolve_ip {
            resolve_ips(&ip_addr, community)
        } else {
            None
        };

        topology::build(&args.ip_address, mac_ports, ip_macs)
    };

    match args.display.as_str() {
        "ascii" => {
            display_ascii::render(&node);
        }
        "tui" => {
            #[cfg(feature = "tui")]
            {
                display_tui::run(node).map_err(AppError::IOError)?;
            }
            #[cfg(not(feature = "tui"))]
            {
                eprintln!("TUI mode requires the 'tui' feature. Rebuild with:");
                eprintln!("  cargo build --features tui");
                return Ok(());
            }
        }
        _ => {
            print_table(&node, &args.sort_by);
        }
    }

    Ok(())
}

fn print_table(node: &topology::NetworkNode, sort_by_col: &str) {
    print_table_recursive(node, sort_by_col, 0);
}

fn print_table_recursive(node: &topology::NetworkNode, sort_by_col: &str, depth: usize) {
    let indent = "  ".repeat(depth);

    if depth > 0 {
        println!();
    }
    println!("{}Switch: {}", indent, node.switch_ip);
    if let Some(ref descr) = node.sys_descr {
        println!("{}  ({})", indent, descr);
    }
    println!("{}{: <5}\t{: <18}\t{}", indent, "port", "mac", "ip");

    let mut rows: Vec<(u32, String, Option<String>, Option<Box<topology::NetworkNode>>)> = Vec::new();
    for port in &node.ports {
        for device in &port.devices {
            rows.push((
                port.port_number,
                device.mac.clone(),
                device.ip.clone(),
                device.child_switch.clone(),
            ));
        }
    }

    match sort_by_col {
        "port" => rows.sort_by_key(|r| r.0),
        _ => rows.sort_by(|a, b| a.1.cmp(&b.1)),
    }

    for (port, mac, ip, child) in &rows {
        let ip_str = ip.as_deref().unwrap_or("");
        let marker = if child.is_some() { " [SWITCH]" } else { "" };
        println!("{}{: <5}\t{: <18}\t{}{}", indent, port, mac, ip_str, marker);
    }

    // Recurse into child switches
    for (_, _, _, child) in &rows {
        if let Some(child_node) = child {
            print_table_recursive(child_node, sort_by_col, depth + 1);
        }
    }
}

/// Try the newer ipNetToPhysicalPhysAddress first, fall back to the classic ipNetToMediaPhysAddress.
pub fn resolve_ips(target: &str, community: &str) -> Option<Vec<mapper::IpMac>> {
    // Try newer MIB first
    if let Ok(r) = mapper::get_ip_to_mac(IP_NET_TO_PHYSICAL_PHYS_ADDRESS, target, community) {
        if !r.is_empty() {
            return Some(r);
        }
    }
    // Fall back to classic ARP table
    match mapper::get_ip_to_mac_legacy(IP_NET_TO_MEDIA_PHYS_ADDRESS, target, community) {
        Ok(r) if !r.is_empty() => Some(r),
        Ok(_) => {
            eprintln!("Warning: no IP-to-MAC entries found on this device");
            None
        }
        Err(e) => {
            eprintln!("Warning: could not resolve IPs: {}", e);
            None
        }
    }
}

pub type AppResult<T> = std::result::Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("SNMP error: {0}")]
    SnmpError(String),
}

impl From<snmp2::Error> for AppError {
    fn from(err: snmp2::Error) -> AppError {
        AppError::SnmpError(format!("{}", err))
    }
}
