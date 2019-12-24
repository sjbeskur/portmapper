use clap::{self, App, Arg};

pub fn process_args<'a>() -> clap::ArgMatches<'a>{
    
    let clap_matches = App::new(crate::APPNAME)
        .version(crate::APPVERSION)
        .author("Advanced Data Machines(tm)")
        .about("Simple switch portmapper utility")
        .arg(Arg::with_name("community")
            .short("c")
            .long("community")
            .value_name("COMMUNITY")
            .help("SNMP v2 community string")
            .takes_value(true)
            .required(true)
        ).arg(Arg::with_name("IPADDRESS")
            .required(true)
            .help("Target IPv4 address to query")
            .index(1)
        ).arg(Arg::with_name("sort_by")
            .short("s")
            .long("sort_by")
            .value_name("SORT")
            .help("Sort by column")
            .takes_value(true)
            .required(false)
            .default_value("mac")
            .possible_values(&["mac","port"])
        )        
        
        .get_matches();    
        clap_matches
}


