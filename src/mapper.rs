extern crate snmp;
use std::time::Duration;
use snmp::{SyncSession};
use eui48::*;
use std::collections::HashMap;

pub fn get_port_macs<'a>(oid: &str, target_ip: &str, community: &str) -> crate::AppResult<Vec<MacPort> >{

    let timeout         = Duration::from_secs(2);
    let non_repeaters   = 0;
    let max_repeaters = 30; // number of items in "system" OID
    
    let results = bulkwalk(oid, target_ip, community, non_repeaters, max_repeaters, timeout)?;

    let mut ret_vec: Vec<MacPort> = Vec::new();        
    let l = oid.len() + 1;
    for (name, port_val) in results {
        if ! name.starts_with(oid){ break; }
        let doted_mac = &name[l..]; // mac is dotd encoded suffix of returned oid        
        let mac = convert_to_mac(doted_mac);
        ret_vec.push( MacPort::new(mac.to_string(MacAddressFormat::Canonical).to_uppercase(), port_val.parse().unwrap() ) );
    }

    Ok(ret_vec)
}

// This function is an implementation of the NET-SNMP utility snmmpbulkwalk
pub fn bulkwalk ( oid: &str, target_ip: &str, community: &str, non_repeaters: u32, max_repeaters: u32, timeout: Duration ) -> crate::AppResult<Vec<(String, String)> >{
    let my_oid = &convert_oid(oid);    
    let mut sess = SyncSession::new(target_ip, community.as_bytes(), Some(timeout), 0)?;    
    bulkwalk_r(&mut sess, my_oid, non_repeaters, max_repeaters)
}

fn bulkwalk_r(sess: &mut SyncSession, oid: &[u32], non_repeaters: u32, max_repeaters: u32 ) -> crate::AppResult< Vec<(String, String)> >{
    let prefix = oid;
    let mut last_oid = prefix;

    let mut result_list: Vec<(String, String)> = Vec::new();
    let mut mem_buff: [u32; 128] = [0; 128];    
    loop{
        match sess.getbulk(&[last_oid], non_repeaters, max_repeaters){        
            Ok(resp) => {
                for (curr_oid, val) in resp.varbinds{
                    let mut oid_buff: [u32; 128] = [0; 128];
                    let coid = curr_oid.read_name(&mut oid_buff).unwrap();
                    if coid.starts_with(prefix) { //if(coid[0..prefix_len] != (*prefix)[..]) {
                        last_oid = curr_oid.read_name(&mut mem_buff).unwrap();

                        let v = format!("{:?}", val);
                        let v: String = v.split(":").skip(1).collect(); // just take the value not the type
                        let v = v.trim();
                        if v.len() < 1 { continue; }
                        let key = format!("{}",curr_oid);
                        //result_map.insert(key, v.to_owned()) ;
                        result_list.push((key, v.to_owned()));
                    }else{
                        return Ok(result_list)
                    }
                }
            },
            Err(e) => {
                return Err(crate::AppError::SnmpError("bulkwalk error has occured".to_string()));
            }
        };
    }

}


#[derive(Clone)]
pub struct MacPort {
    pub mac: String, 
    pub port: u32
}

impl MacPort{
    pub fn new(mac: String, port: u32) -> Self{
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








const TEST_TARGET: &str = "<device_ip>:161";
const TEST_COMMUNITY: &str = "public";

#[test]
fn mac_to_string_test(){
    
    let test = "184.39.235.100.148.192";
    println!("--->{}", convert_to_hex(test));
    let mac = eui48::MacAddress::new([184,39,235,100,148,192]);
        
    assert_eq!(true, true);    
}


#[test]
fn snmp_bulkwalk_test(){
    let ifDesr = "1.3.6.1.2.1.2.2.1.2".to_owned();       // ifDescr
    let non_repeaters   = 0;
    let max_repetitions = 10; // number of items in "system" OID
    let r = self::bulkwalk(&ifDesr ,TEST_TARGET, TEST_COMMUNITY, non_repeaters, max_repetitions, Duration::from_secs(3)).unwrap();
    for (oid, val) in r{
        println!("{}  {}", oid, val);
    }
    assert_eq!(true, true);    
}

#[test]
fn snmp_bulkwalk_macports_test(){
    let macPorts = "1.3.6.1.2.1.17.4.3.1.2".to_owned();
    let non_repeaters   = 0;
    let max_repetitions = 10; // number of items in "system" OID
    let r = self::bulkwalk(&macPorts ,TEST_TARGET, TEST_COMMUNITY, non_repeaters, max_repetitions, Duration::from_secs(3)).unwrap();
    for (oid, val) in r{
        println!("{}  {}", oid, val);
    }
    assert_eq!(true, true);    
}



#[test]
fn to_do(){
    let add_some_test = true;
    assert_eq!(add_some_test, true);
}