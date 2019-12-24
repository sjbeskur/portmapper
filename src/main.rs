extern crate clap;
extern crate eui48;
#[macro_use] extern crate failure;

const APPVERSION: &str = env!("CARGO_PKG_VERSION");
const APPNAME: &str = env!("CARGO_PKG_NAME");

mod cli;
mod mapper;

// snmpbulkwalk -c ggc_ro -v 2c <ip_or_hostname> 1.3.6.1.2.1.17.4.3.1.2

// BRIDGE-MIB::dot1dTpFdbPort
const DOT1D_TP_FDB_PORT: &str = "1.3.6.1.2.1.17.4.3.1.2";

fn main() -> AppResult<()> {
    let matches = cli::process_args();
    
    let community = matches.value_of("community").expect("Invalid SNMP Community string");
    let ip_addr = matches.value_of("IPADDRESS").expect("Invalid IPv4 address");
    let ip_addr = format!("{}:161",ip_addr);

    let _ = match mapper::get_port_macs(DOT1D_TP_FDB_PORT, &ip_addr, community){
        Ok(r) => { r },
        Err(e) => { 
            println!("{}",failure::err_msg(e));
        }
    };

    Ok(())
}


pub type AppResult<T> = std::result::Result<T, AppError>;


#[derive(Fail, Debug)]
pub enum AppError {

    #[fail(display = "IO Error: {}", _0)]
    IOError(String),

    #[fail(display = "SNMP request failed for target: {}", _0)]
    SnmpError(String),

}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> AppError {
        AppError::IOError(err.to_string())
    }
}

impl From<snmp::SnmpError> for AppError{
    fn from( err: snmp::SnmpError) -> AppError{
        AppError::SnmpError( format!("{:?}", err ) )
    }
}


