use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum ProxyType {
    Udp,
    Tcp,
}

#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum LoadBalancerType {
    RoundRobin,
    LeastConnection,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[arg(short, long, help = "The port to listen")]
    pub port: u16,
    #[arg(short, long, help = "The list of backend servers")]
    pub servers: Vec<String>,
    #[arg(
        short = 't',
        long,
        value_enum,
        help = "The protocol to use for the proxy"
    )]
    pub proxy_type: ProxyType,
    #[arg(short, long, help = "The type of load balancer algorithm")]
    pub load_balancer_type: LoadBalancerType,
}
