use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
pub enum ServerType {
    Tcp,
    Udp,
    Http,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[arg(short, long, value_enum, help = "The type of server to start")]
    pub server_type: ServerType,
    #[arg(short, long, help = "The port to listen")]
    pub port: u16,
}
