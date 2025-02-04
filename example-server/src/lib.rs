pub mod clap;
pub mod server;

use ::clap::Parser;
use clap::{CliArgs, ServerType};
use server::{HttpServer, Serverable, TcpServer, UdpServer};
use std::error::Error;
use tracing::{debug, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

/// Runs the application
///
/// # Errors
///
/// Any error that might occur
pub async fn run() -> Result<(), Box<dyn Error>> {
    // Set log and trace level
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(filter).init();
    // Parse arguments and build config
    let args = CliArgs::parse();
    debug!("parsed cli arguments: {:?}", args);
    // Run the desired server type
    let server: Box<dyn Serverable> = match args.server_type {
        ServerType::Udp => Box::new(UdpServer::new(args.port)),
        ServerType::Tcp => Box::new(TcpServer::new(args.port)),
        ServerType::Http => Box::new(HttpServer::new(args.port)),
    };
    server.run().await
}
