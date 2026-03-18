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

    /// Display mode
    #[arg(short, long, default_value = "table", value_parser = ["table", "ascii", "tui"])]
    pub display: String,

    /// Also resolve IP addresses via ipNetToPhysicalPhysAddress
    #[arg(long)]
    pub resolve_ip: bool,

    /// Recursively discover SNMP-enabled switches (implies --resolve-ip)
    #[arg(short, long)]
    pub recursive: bool,

    /// Maximum recursion depth for discovery
    #[arg(long, default_value = "3")]
    pub max_depth: u32,
}

pub fn process_args() -> Args {
    Args::parse()
}
