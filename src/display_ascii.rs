use crate::topology::NetworkNode;

pub fn render(node: &NetworkNode) {
    render_node(node, "", true);
}

fn render_node(node: &NetworkNode, prefix: &str, is_root: bool) {
    // Print switch header
    let descr = node.sys_descr.as_deref().unwrap_or("");
    if is_root {
        println!("[{}] {}", node.switch_ip, descr);
    } else {
        println!("{}[{}] {}", prefix, node.switch_ip, descr);
    }

    let port_count = node.ports.len();
    if port_count == 0 {
        println!("{}(no ports discovered)", tree_prefix(prefix, true));
        return;
    }

    for (port_idx, port) in node.ports.iter().enumerate() {
        let is_last_port = port_idx == port_count - 1;
        let port_connector = if is_last_port { "└── " } else { "├── " };
        let port_continuation = if is_last_port { "    " } else { "│   " };

        let device_summary = format!(
            "{} device{}",
            port.devices.len(),
            if port.devices.len() == 1 { "" } else { "s" }
        );
        println!(
            "{}{}Port {} ({})",
            prefix, port_connector, port.port_number, device_summary
        );

        let device_count = port.devices.len();
        for (dev_idx, device) in port.devices.iter().enumerate() {
            let is_last_device = dev_idx == device_count - 1;
            let dev_connector = if is_last_device { "└── " } else { "├── " };
            let dev_continuation = if is_last_device { "    " } else { "│   " };

            let ip_str = device.ip.as_ref().map(|ip| format!(" ({})", ip)).unwrap_or_default();
            let switch_marker = if device.is_switch() { " [SWITCH]" } else { "" };

            println!(
                "{}{}{}{}{}{}",
                prefix, port_continuation, dev_connector, device.mac, ip_str, switch_marker
            );

            // If this device is a child switch, render it recursively
            if let Some(ref child) = device.child_switch {
                let child_prefix = format!("{}{}{}", prefix, port_continuation, dev_continuation);
                render_node(child, &child_prefix, false);
            }
        }
    }
}

fn tree_prefix(prefix: &str, is_last: bool) -> String {
    format!("{}{}", prefix, if is_last { "    " } else { "│   " })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::{NetworkNode, NetworkPort, NetworkDevice};

    fn capture_render(node: &NetworkNode) -> String {
        // We can't easily capture println! output, so test build_lines logic instead
        // For now just verify it doesn't panic
        render(node);
        String::new()
    }

    #[test]
    fn test_single_switch_no_panic() {
        let node = NetworkNode {
            switch_ip: "10.0.0.1".into(),
            sys_descr: Some("Test Switch".into()),
            ports: vec![
                NetworkPort {
                    port_number: 1,
                    devices: vec![
                        NetworkDevice { mac: "AA:BB:CC:DD:EE:01".into(), ip: None, child_switch: None },
                    ],
                },
            ],
        };
        capture_render(&node);
    }

    #[test]
    fn test_recursive_tree_no_panic() {
        let child = NetworkNode {
            switch_ip: "10.0.0.2".into(),
            sys_descr: Some("Child Switch".into()),
            ports: vec![
                NetworkPort {
                    port_number: 1,
                    devices: vec![
                        NetworkDevice { mac: "11:22:33:44:55:66".into(), ip: Some("10.0.0.50".into()), child_switch: None },
                    ],
                },
            ],
        };
        let root = NetworkNode {
            switch_ip: "10.0.0.1".into(),
            sys_descr: Some("Root Switch".into()),
            ports: vec![
                NetworkPort {
                    port_number: 1,
                    devices: vec![
                        NetworkDevice { mac: "AA:BB:CC:DD:EE:01".into(), ip: None, child_switch: None },
                    ],
                },
                NetworkPort {
                    port_number: 2,
                    devices: vec![
                        NetworkDevice {
                            mac: "BB:CC:DD:EE:FF:02".into(),
                            ip: Some("10.0.0.2".into()),
                            child_switch: Some(Box::new(child)),
                        },
                    ],
                },
            ],
        };
        capture_render(&root);
    }

    #[test]
    fn test_empty_topology() {
        let node = NetworkNode {
            switch_ip: "10.0.0.1".into(),
            sys_descr: None,
            ports: vec![],
        };
        capture_render(&node);
    }

    #[test]
    fn test_multiple_devices_per_port() {
        let node = NetworkNode {
            switch_ip: "10.0.0.1".into(),
            sys_descr: None,
            ports: vec![
                NetworkPort {
                    port_number: 3,
                    devices: vec![
                        NetworkDevice { mac: "AA:BB:CC:DD:EE:01".into(), ip: None, child_switch: None },
                        NetworkDevice { mac: "AA:BB:CC:DD:EE:02".into(), ip: None, child_switch: None },
                        NetworkDevice { mac: "AA:BB:CC:DD:EE:03".into(), ip: None, child_switch: None },
                    ],
                },
            ],
        };
        capture_render(&node);
    }
}
