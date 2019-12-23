extern crate clap;
extern crate eui48;

const APPVERSION: &str = env!("CARGO_PKG_VERSION");
const APPNAME: &str = env!("CARGO_PKG_NAME");

mod cli;
mod mapper;

// snmpbulkwalk -c ggc_ro -v 2c 10.80.4.14 
//       1.3.6.1.2.1.17.4.3.1.2

// BRIDGE-MIB::dot1dTpFdbPort
const DOT1D_TP_FDB_PORT: &str = "1.3.6.1.2.1.17.4.3.1.2";

fn main() {
    let matches = cli::process_args();
    
    let community = matches.value_of("community").unwrap();
    let ip_addr = matches.value_of("IPADDRESS").unwrap();
    let ip_addr = format!("{}:161",ip_addr);

    mapper::get_port_macs(DOT1D_TP_FDB_PORT, &ip_addr, community);

}


