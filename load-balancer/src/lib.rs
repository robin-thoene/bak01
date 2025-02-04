pub mod clap;
pub mod server;

use ::clap::Parser;
use clap::{CliArgs, ProxyType};
use server::{Proxy, TcpProxy, UdpProxy};
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
    // Ensure that at least one backend to proxy is provided
    if args.servers.is_empty() {
        panic!("At least one backend server must be provided");
    }
    // Run the desired proxy type
    let proxy: Box<dyn Proxy> = match args.proxy_type {
        ProxyType::Udp => Box::new(UdpProxy::new(args.port, args.servers)),
        ProxyType::Tcp => Box::new(TcpProxy::new(args.port, args.servers)),
    };
    proxy.run().await
}
