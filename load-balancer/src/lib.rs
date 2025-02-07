pub mod clap;
pub mod load_balancer;
pub mod reverse_proxy;

use ::clap::Parser;
use clap::{CliArgs, ProxyType};
use load_balancer::{LoadBalancer, RoundRobinLoadBalancer};
use reverse_proxy::{Proxy, TcpProxy, UdpProxy};
use std::{error::Error, sync::Arc};
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
    let round = Arc::new(RoundRobinLoadBalancer::new(args.servers));
    let proxy: Box<dyn Proxy> = match args.proxy_type {
        ProxyType::Udp => Box::new(UdpProxy::new(args.port, round)),
        ProxyType::Tcp => Box::new(TcpProxy::new(args.port, round)),
    };
    proxy.run().await
}
