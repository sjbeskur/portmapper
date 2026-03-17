use thiserror::Error;

const APPVERSION: &str = env!("CARGO_PKG_VERSION");
const APPNAME: &str = env!("CARGO_PKG_NAME");

mod cli;
mod mapper;

// snmpbulkwalk -c ggc_ro -v 2c <ip_or_hostname> 1.3.6.1.2.1.17.4.3.1.2
// BRIDGE-MIB::dot1dTpFdbPort
const DOT1D_TP_FDB_PORT: &str = "1.3.6.1.2.1.17.4.3.1.2";

// snmpbulkwalk -c ggc_ro -v 2c 10.80.4.14 1.3.6.1.2.1.4.35 -m IP-MIB
// IP-MIB::ipNetToPhysicalPhysAddress
const IP_NET_TO_PHYSICAL_PHYS_ADDRESS: &str = "1.3.6.1.2.1.4.35";

fn main() -> AppResult<()> {
    let matches = cli::process_args();

    let sort_by = matches.sort_by.as_str();
    let community = &matches.community;
    let ip_addr = format!("{}:161", &matches.ip_address);

    match mapper::get_port_macs(DOT1D_TP_FDB_PORT, &ip_addr, community) {
        Ok(mut r) => {
            print_results(&mut r, sort_by);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    };
    Ok(())
}

fn print_results(list: &mut Vec<mapper::MacPort>, sort_by_col: &str) {
    println!("{0: <5}\t{1: <18}", "port", "mac");
    match sort_by_col {
        "port" => list.sort_by_key(|k| k.port),
        _ => list.sort_by_key(|k| k.mac.clone()),
    }

    for i in list {
        println!("{0: <5}\t{1: <18}", i.port, i.mac);
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
