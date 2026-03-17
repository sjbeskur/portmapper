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

#[derive(Clone)]
pub struct MacPort {
    pub mac: String,
    pub port: u32,
}

impl MacPort {
    pub fn new(mac: String, port: u32) -> Self {
        Self { mac, port }
    }
}

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
