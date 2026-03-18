use std::collections::HashMap;
use crate::mapper::{MacPort, IpMac};

#[derive(Clone, Debug)]
pub struct NetworkNode {
    pub switch_ip: String,
    pub sys_descr: Option<String>,
    pub ports: Vec<NetworkPort>,
}

#[derive(Clone, Debug)]
pub struct NetworkPort {
    pub port_number: u32,
    pub devices: Vec<NetworkDevice>,
}

#[derive(Clone, Debug)]
pub struct NetworkDevice {
    pub mac: String,
    pub ip: Option<String>,
    pub child_switch: Option<Box<NetworkNode>>,
}

impl NetworkDevice {
    pub fn is_switch(&self) -> bool {
        self.child_switch.is_some()
    }
}

/// Build a single-level NetworkNode from MAC/port and optional IP data.
/// Used for non-recursive mode and as a building block for recursive discovery.
pub fn build(switch_ip: &str, mac_ports: Vec<MacPort>, ip_macs: Option<Vec<IpMac>>) -> NetworkNode {
    let ip_by_mac: HashMap<String, String> = ip_macs
        .unwrap_or_default()
        .into_iter()
        .map(|im| (im.mac.to_uppercase(), im.ip))
        .collect();

    let mut port_map: HashMap<u32, Vec<NetworkDevice>> = HashMap::new();
    for mp in mac_ports {
        let mac_upper = mp.mac.to_uppercase();
        let ip = ip_by_mac.get(&mac_upper).cloned();
        port_map
            .entry(mp.port)
            .or_default()
            .push(NetworkDevice {
                mac: mac_upper,
                ip,
                child_switch: None,
            });
    }

    let mut ports: Vec<NetworkPort> = port_map
        .into_iter()
        .map(|(port_number, mut devices)| {
            devices.sort_by(|a, b| a.mac.cmp(&b.mac));
            NetworkPort { port_number, devices }
        })
        .collect();
    ports.sort_by_key(|p| p.port_number);

    NetworkNode {
        switch_ip: switch_ip.to_string(),
        sys_descr: None,
        ports,
    }
}

/// Count total devices across all ports (non-recursive, just this node).
#[allow(dead_code)]
pub fn device_count(node: &NetworkNode) -> usize {
    node.ports.iter().map(|p| p.devices.len()).sum()
}

/// Count total switches in the tree (including root).
#[allow(dead_code)]
pub fn switch_count(node: &NetworkNode) -> usize {
    let mut count = 1;
    for port in &node.ports {
        for device in &port.devices {
            if let Some(ref child) = device.child_switch {
                count += switch_count(child);
            }
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_groups_by_port() {
        let mac_ports = vec![
            MacPort::new("AA:BB:CC:DD:EE:01".into(), 1),
            MacPort::new("AA:BB:CC:DD:EE:02".into(), 1),
            MacPort::new("AA:BB:CC:DD:EE:03".into(), 2),
        ];
        let node = build("10.0.0.1", mac_ports, None);
        assert_eq!(node.ports.len(), 2);
        assert_eq!(node.ports[0].port_number, 1);
        assert_eq!(node.ports[0].devices.len(), 2);
        assert_eq!(node.ports[1].port_number, 2);
        assert_eq!(node.ports[1].devices.len(), 1);
    }

    #[test]
    fn test_build_with_ip_enrichment() {
        let mac_ports = vec![
            MacPort::new("AA:BB:CC:DD:EE:01".into(), 1),
        ];
        let ip_macs = vec![
            IpMac { ip: "10.0.0.100".into(), mac: "AA:BB:CC:DD:EE:01".into() },
        ];
        let node = build("10.0.0.1", mac_ports, Some(ip_macs));
        assert_eq!(node.ports[0].devices[0].ip, Some("10.0.0.100".into()));
    }

    #[test]
    fn test_build_no_ip_match() {
        let mac_ports = vec![
            MacPort::new("AA:BB:CC:DD:EE:01".into(), 1),
        ];
        let ip_macs = vec![
            IpMac { ip: "10.0.0.200".into(), mac: "FF:FF:FF:FF:FF:FF".into() },
        ];
        let node = build("10.0.0.1", mac_ports, Some(ip_macs));
        assert_eq!(node.ports[0].devices[0].ip, None);
    }

    #[test]
    fn test_device_count() {
        let node = NetworkNode {
            switch_ip: "10.0.0.1".into(),
            sys_descr: None,
            ports: vec![
                NetworkPort {
                    port_number: 1,
                    devices: vec![
                        NetworkDevice { mac: "AA:BB:CC:DD:EE:01".into(), ip: None, child_switch: None },
                        NetworkDevice { mac: "AA:BB:CC:DD:EE:02".into(), ip: None, child_switch: None },
                    ],
                },
                NetworkPort {
                    port_number: 2,
                    devices: vec![
                        NetworkDevice { mac: "AA:BB:CC:DD:EE:03".into(), ip: None, child_switch: None },
                    ],
                },
            ],
        };
        assert_eq!(device_count(&node), 3);
    }

    #[test]
    fn test_switch_count_with_children() {
        let child = NetworkNode {
            switch_ip: "10.0.0.2".into(),
            sys_descr: None,
            ports: vec![],
        };
        let root = NetworkNode {
            switch_ip: "10.0.0.1".into(),
            sys_descr: None,
            ports: vec![
                NetworkPort {
                    port_number: 1,
                    devices: vec![
                        NetworkDevice {
                            mac: "AA:BB:CC:DD:EE:01".into(),
                            ip: Some("10.0.0.2".into()),
                            child_switch: Some(Box::new(child)),
                        },
                    ],
                },
            ],
        };
        assert_eq!(switch_count(&root), 2);
    }
}
