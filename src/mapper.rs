use std::time::Duration;
use snmp2::{SyncSession, Oid, Value};
use eui48::MacAddress;

pub fn get_port_macs(oid: &str, target_ip: &str, community: &str) -> crate::AppResult<Vec<MacPort>> {
    let timeout = Duration::from_secs(2);
    let non_repeaters = 0;
    let max_repeaters = 30;

    let results = bulkwalk(oid, target_ip, community, non_repeaters, max_repeaters, timeout)?;

    let mut ret_vec: Vec<MacPort> = Vec::new();
    let prefix = format!("{}.", oid);
    for (name, port_val) in results {
        if !name.starts_with(&prefix) {
            break;
        }
        let dotted_mac = &name[prefix.len()..];
        let mac = convert_to_mac(dotted_mac);
        let port: u32 = port_val.parse().unwrap_or(0);
        ret_vec.push(MacPort::new(
            mac.to_hex_string().to_uppercase(),
            port,
        ));
    }

    Ok(ret_vec)
}

/// SNMP bulk-walk implementation using GETBULK requests
pub fn bulkwalk(
    oid: &str,
    target_ip: &str,
    community: &str,
    non_repeaters: u32,
    max_repeaters: u32,
    timeout: Duration,
) -> crate::AppResult<Vec<(String, String)>> {
    let mut sess = SyncSession::new_v2c(target_ip, community.as_bytes(), Some(timeout), 0)?;
    let prefix_oid: Oid<'static> = oid.parse().map_err(|e| crate::AppError::SnmpError(format!("Invalid OID '{}': {:?}", oid, e)))?;

    let mut result_list: Vec<(String, String)> = Vec::new();
    let mut last_oid = prefix_oid.clone();

    loop {
        let resp = sess.getbulk(&[&last_oid], non_repeaters, max_repeaters)?;
        let mut done = false;

        for (curr_oid, val) in resp.varbinds {
            let curr_oid_str = curr_oid.to_string();
            let prefix_str = prefix_oid.to_string();

            if !curr_oid_str.starts_with(&prefix_str) {
                done = true;
                break;
            }

            if matches!(val, Value::EndOfMibView) {
                done = true;
                break;
            }

            let v = format_value(&val);
            if !v.is_empty() {
                result_list.push((curr_oid_str, v));
            }

            last_oid = curr_oid.to_owned();
        }

        if done {
            break;
        }
    }

    Ok(result_list)
}

fn format_value(val: &Value) -> String {
    match val {
        Value::Integer(n) => n.to_string(),
        Value::Unsigned32(n) => n.to_string(),
        Value::Counter32(n) => n.to_string(),
        Value::Counter64(n) => n.to_string(),
        Value::Timeticks(n) => n.to_string(),
        Value::OctetString(bytes) => String::from_utf8_lossy(bytes).to_string(),
        Value::IpAddress(addr) => format!("{}.{}.{}.{}", addr[0], addr[1], addr[2], addr[3]),
        Value::ObjectIdentifier(oid) => oid.to_string(),
        _ => format!("{:?}", val),
    }
}

#[derive(Clone, Debug)]
pub struct MacPort {
    pub mac: String,
    pub port: u32,
}

impl MacPort {
    pub fn new(mac: String, port: u32) -> Self {
        Self { mac, port }
    }
}

#[derive(Clone, Debug)]
pub struct IpMac {
    pub ip: String,
    pub mac: String,
}

/// Query ipNetToPhysicalPhysAddress to get IP-to-MAC mappings.
/// The OID suffix format is: ifIndex.ipAddressType.ipAddress...
/// The value is an OctetString containing the raw MAC bytes.
pub fn get_ip_to_mac(oid: &str, target_ip: &str, community: &str) -> crate::AppResult<Vec<IpMac>> {
    let timeout = Duration::from_secs(2);
    let results = bulkwalk_raw(oid, target_ip, community, 0, 30, timeout)?;

    let mut ret_vec: Vec<IpMac> = Vec::new();
    let prefix = format!("{}.", oid);

    for (name, raw_value) in results {
        if !name.starts_with(&prefix) {
            break;
        }
        let suffix = &name[prefix.len()..];
        // suffix format: ifIndex.ipAddressType.ipAddress...
        // ipAddressType: 1=IPv4, 2=IPv6
        let parts: Vec<&str> = suffix.splitn(3, '.').collect();
        if parts.len() < 3 {
            continue;
        }
        let addr_type: u32 = parts[1].parse().unwrap_or(0);
        if addr_type != 1 {
            // Skip non-IPv4 for now
            continue;
        }
        let ip_str = parts[2].to_string();
        // raw_value should be 6 bytes for a MAC address
        if raw_value.len() == 6 {
            let mac = format!(
                "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                raw_value[0], raw_value[1], raw_value[2],
                raw_value[3], raw_value[4], raw_value[5]
            );
            ret_vec.push(IpMac { ip: ip_str, mac });
        }
    }

    Ok(ret_vec)
}

/// Query the classic ipNetToMediaPhysAddress (RFC 1213) ARP table.
/// OID suffix format: ifIndex.ipAddress (e.g., 1.10.0.0.5)
/// Value: 6-byte OctetString containing the MAC address.
pub fn get_ip_to_mac_legacy(oid: &str, target_ip: &str, community: &str) -> crate::AppResult<Vec<IpMac>> {
    let timeout = Duration::from_secs(2);
    let results = bulkwalk_raw(oid, target_ip, community, 0, 30, timeout)?;

    let mut ret_vec: Vec<IpMac> = Vec::new();
    let prefix = format!("{}.", oid);

    for (name, raw_value) in results {
        if !name.starts_with(&prefix) {
            break;
        }
        let suffix = &name[prefix.len()..];
        // suffix format: ifIndex.a.b.c.d  where a.b.c.d is the IP
        // Split off the first component (ifIndex), rest is the IP
        if let Some(dot_pos) = suffix.find('.') {
            let ip_str = &suffix[dot_pos + 1..];
            // Validate it looks like an IPv4 address (4 octets)
            let octets: Vec<&str> = ip_str.split('.').collect();
            if octets.len() == 4 && raw_value.len() == 6 {
                let mac = format!(
                    "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                    raw_value[0], raw_value[1], raw_value[2],
                    raw_value[3], raw_value[4], raw_value[5]
                );
                ret_vec.push(IpMac { ip: ip_str.to_string(), mac });
            }
        }
    }

    Ok(ret_vec)
}

/// Like bulkwalk but returns raw bytes for OctetString values
fn bulkwalk_raw(
    oid: &str,
    target_ip: &str,
    community: &str,
    non_repeaters: u32,
    max_repeaters: u32,
    timeout: Duration,
) -> crate::AppResult<Vec<(String, Vec<u8>)>> {
    let mut sess = SyncSession::new_v2c(target_ip, community.as_bytes(), Some(timeout), 0)?;
    let prefix_oid: Oid<'static> = oid.parse().map_err(|e| crate::AppError::SnmpError(format!("Invalid OID '{}': {:?}", oid, e)))?;

    let mut result_list: Vec<(String, Vec<u8>)> = Vec::new();
    let mut last_oid = prefix_oid.clone();

    loop {
        let resp = sess.getbulk(&[&last_oid], non_repeaters, max_repeaters)?;
        let mut done = false;

        for (curr_oid, val) in resp.varbinds {
            let curr_oid_str = curr_oid.to_string();
            let prefix_str = prefix_oid.to_string();

            if !curr_oid_str.starts_with(&prefix_str) {
                done = true;
                break;
            }

            if matches!(val, Value::EndOfMibView) {
                done = true;
                break;
            }

            if let Value::OctetString(bytes) = val {
                result_list.push((curr_oid_str, bytes.to_vec()));
            }

            last_oid = curr_oid.to_owned();
        }

        if done {
            break;
        }
    }

    Ok(result_list)
}

/// Quick SNMP probe: tries a GET on sysDescr. Returns Some(description) if the host responds.
pub fn snmp_probe(target_ip: &str, community: &str) -> Option<String> {
    let timeout = Duration::from_millis(500);
    let mut sess = SyncSession::new_v2c(target_ip, community.as_bytes(), Some(timeout), 0).ok()?;
    // sysDescr.0
    let oid: Oid<'static> = "1.3.6.1.2.1.1.1.0".parse().ok()?;
    let resp = sess.get(&oid).ok()?;
    for (_oid, val) in resp.varbinds {
        if let Value::OctetString(bytes) = val {
            return Some(String::from_utf8_lossy(bytes).to_string());
        }
    }
    None
}

#[allow(dead_code)]
pub fn convert_oid(oid: &str) -> Vec<u32> {
    oid.split('.')
        .map(|c| c.parse::<u32>().unwrap())
        .collect()
}

pub fn convert_to_mac(dotted: &str) -> MacAddress {
    let bytes: Vec<u8> = dotted
        .split('.')
        .map(|n| n.parse::<u8>().unwrap())
        .collect();

    let mut buffer: [u8; 6] = Default::default();
    buffer.copy_from_slice(&bytes[0..6]);
    MacAddress::new(buffer)
}

#[allow(dead_code)]
pub fn convert_to_hex(dotted: &str) -> String {
    dotted
        .split('.')
        .map(|n| format!("{:02X}", n.parse::<u32>().unwrap()))
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mac_to_string_test() {
        let test = "184.39.235.100.148.192";
        let hex = convert_to_hex(test);
        assert_eq!(hex, "B827EB6494C0");

        let mac = convert_to_mac(test);
        assert_eq!(mac.to_hex_string().to_uppercase(), "B8:27:EB:64:94:C0");
    }

    #[test]
    fn convert_oid_test() {
        let oid = convert_oid("1.3.6.1.2.1");
        assert_eq!(oid, vec![1, 3, 6, 1, 2, 1]);
    }

    #[test]
    fn convert_to_hex_test() {
        assert_eq!(convert_to_hex("255.0.128"), "FF0080");
    }
}
