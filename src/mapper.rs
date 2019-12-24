extern crate snmp;
use std::time::Duration;
use snmp::{SyncSession};
use eui48::*;

pub fn get_port_macs<'a>(oid: &str, target_ip: &str, community: &str) -> crate::AppResult<Vec<VarBind> >{

    let system_oid      = &convert_oid(oid);
    let community = community.as_bytes();
    let timeout         = Duration::from_secs(2);

    let non_repeaters   = 0;
    let max_repetitions = 30; // number of items in "system" OID
    
    let mut sess = SyncSession::new(target_ip, community, Some(timeout), 0)?;

    let response = match sess.getbulk(&[system_oid], non_repeaters, max_repetitions){
        Ok(resp) => { resp },
        Err(_) => { 
            return Err(crate::AppError::SnmpError(target_ip.to_string()));
        }
    };

    let mut ret_vec: Vec<VarBind> = Vec::new();        

    let l = oid.len() + 1;
    for (name, port_val) in response.varbinds {
        let s = format!("{}", name);
        let doted_mac = &s[l..]; // mac is dotd encoded suffix of returned oid

        //let hex_mac = convert_to_hex(doted_mac);
        //println!("{} - {} - port={:?}", hex_mac, hex_mac.len(), port_val);
        
        let val = format!("{:?}", port_val);
        let port: Vec<&str> = val.split(":").collect();
        let mac = convert_to_mac(doted_mac);

        ret_vec.push( VarBind::new(mac.to_string(MacAddressFormat::Canonical).to_uppercase(), port[1].trim().into()) );
    }

    Ok(ret_vec)
}

#[derive(Clone)]
pub struct VarBind {
    pub mac: String, 
    pub port: String
}

impl VarBind{
    pub fn new(mac: String, port: String) -> Self{
        Self{ mac, port }
    }
}


pub fn convert_oid<S:Into<String>>(oid: S) -> Vec<u32>{
    oid.into().split(".").into_iter().map(|c| c.parse::<u32>().unwrap()).collect()
}


pub fn convert_to_mac<S:Into<String>>(dotd: S) -> eui48::MacAddress {
    let r = dotd.into().split(".").into_iter().map( |n| n.parse::<u8>().unwrap()).collect::<Vec<u8>>();        
    let mut buffer: [u8;6] = Default::default();
    buffer.copy_from_slice(&r[0..6]);
    eui48::MacAddress::new(buffer)
}

#[allow(dead_code)]
pub fn convert_to_hex<S:Into<String>>(dotd: S) -> String{
    return  dotd.into().split(".").into_iter().map(
        |n| format!("{:02X}", n.parse::<u32>().unwrap())
    ).collect::<String>()
}


#[test]
fn mac_to_string_test(){
    
    let test = "184.39.235.100.148.192";
    println!("--->{}", convert_to_hex(test));
    let mac = eui48::MacAddress::new([184,39,235,100,148,192]);
        
    assert_eq!(true, true);    
}

#[test]
fn to_do(){
    let add_some_test = true;
    assert_eq!(add_some_test, true);
}