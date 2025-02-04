use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
pub enum ProxyType {
    Udp,
    Tcp,
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
}
