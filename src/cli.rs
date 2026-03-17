use clap::Parser;

#[derive(Parser)]
#[command(name = crate::APPNAME, version = crate::APPVERSION)]
#[command(author = "Advanced Data Machines(tm)")]
#[command(about = "Simple switch portmapper utility")]
pub struct Args {
    /// SNMP v2 community string
    #[arg(short, long)]
    pub community: String,

    /// Target IPv4 address to query
    pub ip_address: String,

    /// Sort by column
    #[arg(short, long, default_value = "mac", value_parser = ["mac", "port"])]
    pub sort_by: String,
}

pub fn process_args() -> Args {
    Args::parse()
}
