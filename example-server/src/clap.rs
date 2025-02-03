use clap::{Parser, ValueEnum};

use crate::server::Protocol;

#[derive(Debug, Clone, ValueEnum)]
pub enum ServerType {
    Tcp,
    Udp,
    Http,
}

impl From<ServerType> for Protocol {
    fn from(value: ServerType) -> Self {
        match value {
            ServerType::Tcp => Protocol::Tcp,
            ServerType::Udp => Protocol::Udp,
            ServerType::Http => Protocol::Http,
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[arg(short, long, value_enum, help = "The type of server to start")]
    pub server_type: ServerType,
    #[arg(short, long, help = "The port to listen")]
    pub port: u16,
}
